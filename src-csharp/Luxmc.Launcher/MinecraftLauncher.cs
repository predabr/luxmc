using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Text;
using System.Text.Json;

namespace Luxmc.Launcher;

public class MinecraftLauncher
{
    public static async Task<LaunchResult> LaunchAsync(LaunchRequest request, bool supervise = true)
    {
        await Task.Yield();

        if (string.IsNullOrWhiteSpace(request.GameDir))
        {
            throw new ArgumentException("Diretório do jogo (gameDir) não especificado.");
        }

        Directory.CreateDirectory(request.GameDir);
        Directory.CreateDirectory(Path.Combine(request.GameDir, "logs"));

        // Detect Java runtime if set to default
        string javaExe = ResolveJavaExecutable(request.JavaPath);

        // Assemble Classpath
        char separator = RuntimeInformation.IsOSPlatform(OSPlatform.Windows) ? ';' : ':';
        var validJars = request.Classpath.Where(File.Exists).ToList();
        if (validJars.Count == 0 && request.Classpath.Count > 0)
        {
            // fallback to verbatim classpath entries
            validJars = request.Classpath;
        }
        string classpathString = string.Join(separator, validJars);

        // Compose Arguments
        var argsList = new List<string>();

        bool hasCpInJvm = request.JvmArgs.Any(a => a == "-cp" || a == "-classpath" || a.StartsWith("-cp=") || a.StartsWith("-classpath="));
        bool hasXmxInJvm = request.JvmArgs.Any(a => a.StartsWith("-Xmx"));
        bool hasXmsInJvm = request.JvmArgs.Any(a => a.StartsWith("-Xms"));

        // 1. Optimized JVM Flags
        int javaMajor = DetectJavaMajorVersion(javaExe);
        var jvmFlags = GCTuner.GenerateOptimizedJvmFlags(request.MemoryMb, request.MinMemoryMb, javaMajor);
        foreach (var flag in jvmFlags)
        {
            if (flag.StartsWith("-Xmx") && hasXmxInJvm) continue;
            if (flag.StartsWith("-Xms") && hasXmsInJvm) continue;
            argsList.Add(flag);
        }

        // 2. Custom JVM Arguments
        foreach (var jvmArg in request.JvmArgs)
        {
            if (!string.IsNullOrWhiteSpace(jvmArg))
            {
                // Filter dangerous flags on Linux
                if (!RuntimeInformation.IsOSPlatform(OSPlatform.OSX) && jvmArg.Contains("-XstartOnFirstThread"))
                {
                    continue;
                }
                argsList.Add(jvmArg);
            }
        }

        // 3. Classpath
        if (!hasCpInJvm && !string.IsNullOrEmpty(classpathString))
        {
            argsList.Add("-cp");
            argsList.Add(classpathString);
        }

        // 4. Main Class
        bool mainClassPresent = argsList.Contains(request.MainClass) || request.GameArgs.Contains(request.MainClass);
        if (!mainClassPresent && !string.IsNullOrWhiteSpace(request.MainClass))
        {
            argsList.Add(request.MainClass);
        }

        // 5. Game Arguments
        bool HasGameArg(string name) => request.GameArgs.Any(a => a.Equals(name, StringComparison.OrdinalIgnoreCase));

        if (!HasGameArg("--username"))
        {
            argsList.Add("--username");
            argsList.Add(string.IsNullOrWhiteSpace(request.Username) ? "Player" : request.Username);
        }

        if (!HasGameArg("--version"))
        {
            argsList.Add("--version");
            argsList.Add(string.IsNullOrWhiteSpace(request.VersionId) ? "1.20.1" : request.VersionId);
        }

        if (!HasGameArg("--gameDir"))
        {
            argsList.Add("--gameDir");
            argsList.Add(request.GameDir);
        }

        if (!HasGameArg("--assetsDir") && !string.IsNullOrEmpty(request.AssetsDir))
        {
            argsList.Add("--assetsDir");
            argsList.Add(request.AssetsDir);
        }

        if (!HasGameArg("--assetIndex") && !string.IsNullOrEmpty(request.AssetIndex))
        {
            argsList.Add("--assetIndex");
            argsList.Add(request.AssetIndex);
        }

        if (!HasGameArg("--uuid"))
        {
            argsList.Add("--uuid");
            argsList.Add(string.IsNullOrWhiteSpace(request.Uuid) ? Guid.NewGuid().ToString("N") : request.Uuid);
        }

        if (!HasGameArg("--accessToken"))
        {
            argsList.Add("--accessToken");
            argsList.Add(string.IsNullOrWhiteSpace(request.AccessToken) ? "-" : request.AccessToken);
        }

        if (!HasGameArg("--userType"))
        {
            argsList.Add("--userType");
            argsList.Add(string.IsNullOrWhiteSpace(request.UserType) ? "mojang" : request.UserType);
        }

        if (!HasGameArg("--versionType"))
        {
            argsList.Add("--versionType");
            argsList.Add(string.IsNullOrWhiteSpace(request.VersionType) ? "Luxmc" : request.VersionType);
        }

        if (!HasGameArg("--server") && !string.IsNullOrWhiteSpace(request.ServerIp))
        {
            argsList.Add("--server");
            argsList.Add(request.ServerIp);

            if (!HasGameArg("--port") && request.ServerPort.HasValue && request.ServerPort.Value > 0)
            {
                argsList.Add("--port");
                argsList.Add(request.ServerPort.Value.ToString());
            }
        }

        if (!HasGameArg("--width") && request.ResolutionWidth > 0 && request.ResolutionHeight > 0)
        {
            argsList.Add("--width");
            argsList.Add(request.ResolutionWidth.ToString());
            argsList.Add("--height");
            argsList.Add(request.ResolutionHeight.ToString());
        }

        if (!HasGameArg("--demo") && request.IsDemo)
        {
            argsList.Add("--demo");
        }

        foreach (var gArg in request.GameArgs)
        {
            if (!string.IsNullOrWhiteSpace(gArg))
            {
                argsList.Add(gArg);
            }
        }

        // Configure Process
        var startInfo = new ProcessStartInfo
        {
            FileName = javaExe,
            WorkingDirectory = request.GameDir,
            UseShellExecute = false,
            RedirectStandardOutput = true,
            RedirectStandardError = true,
            StandardOutputEncoding = Encoding.UTF8,
            StandardErrorEncoding = Encoding.UTF8
        };

        foreach (var arg in argsList)
        {
            startInfo.ArgumentList.Add(arg);
        }

        // Environment variables
        if (request.EnableVulkan == false)
        {
            startInfo.Environment["LUXMC_DISABLE_VULKAN"] = "1";
        }

        if (request.CustomEnv != null)
        {
            foreach (var kv in request.CustomEnv)
            {
                startInfo.Environment[kv.Key] = kv.Value;
            }
        }

        var process = new Process { StartInfo = startInfo, EnableRaisingEvents = true };

        var logBuffer = new List<string>();
        var bufferLock = new object();

        void HandleOutput(string? line, string stream)
        {
            if (string.IsNullOrEmpty(line)) return;

            lock (bufferLock)
            {
                if (logBuffer.Count > 1200)
                {
                    logBuffer.RemoveRange(0, 200);
                }
                logBuffer.Add(line);
            }

            if (stream == "stderr")
            {
                Console.Error.WriteLine(line);
            }
            else
            {
                Console.WriteLine(line);
            }
        }

        process.OutputDataReceived += (_, e) => HandleOutput(e.Data, "stdout");
        process.ErrorDataReceived += (_, e) => HandleOutput(e.Data, "stderr");

        bool started = process.Start();
        if (!started)
        {
            return new LaunchResult
            {
                Success = false,
                Pid = 0,
                Message = "Falha ao iniciar processo Java do Minecraft."
            };
        }

        int pid = process.Id;
        process.BeginOutputReadLine();
        process.BeginErrorReadLine();

        var result = new LaunchResult
        {
            Success = true,
            Pid = pid,
            Message = $"Minecraft iniciado com sucesso (PID: {pid}).",
            CommandLine = $"{javaExe} {string.Join(" ", argsList)}"
        };

        // Notify launch event
        var launchEvent = new
        {
            type = "game-launched",
            pid = pid,
            versionId = request.VersionId,
            profile = request.Username
        };
        Console.WriteLine(JsonSerializer.Serialize(launchEvent));

        if (supervise)
        {
            await process.WaitForExitAsync();
            int exitCode = process.ExitCode;

            string fullLog;
            lock (bufferLock)
            {
                fullLog = string.Join(Environment.NewLine, logBuffer);
            }

            if (exitCode != 0)
            {
                var diagnosis = CrashDoctor.Analyze(fullLog);
                Console.Error.WriteLine($"[Luxmc.CrashDoctor] {JsonSerializer.Serialize(diagnosis)}");
            }
        }

        return result;
    }

    public static string ResolveJavaExecutable(string? specifiedPath)
    {
        if (!string.IsNullOrWhiteSpace(specifiedPath) && (specifiedPath.Contains(Path.DirectorySeparatorChar) || specifiedPath.Contains(Path.AltDirectorySeparatorChar)))
        {
            if (File.Exists(specifiedPath)) return specifiedPath;
        }

        string defaultExe = RuntimeInformation.IsOSPlatform(OSPlatform.Windows) ? "javaw.exe" : "java";

        // Check common paths
        var candidates = new List<string>();
        if (RuntimeInformation.IsOSPlatform(OSPlatform.Linux))
        {
            candidates.Add("/usr/bin/java");
            candidates.Add("/usr/lib/jvm/default-runtime/bin/java");
            candidates.Add(Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".local/share/luxmc/runtime/java-runtime-gamma/bin/java"));
            candidates.Add(Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".local/share/luxmc/runtime/java-runtime-delta/bin/java"));
        }
        else if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
        {
            candidates.Add(@"C:\Program Files\Java\jdk-21\bin\javaw.exe");
            candidates.Add(@"C:\Program Files\Java\jdk-17\bin\javaw.exe");
            candidates.Add(@"C:\Program Files\Eclipse Adoptium\jdk-21\bin\javaw.exe");
            candidates.Add(@"C:\Program Files\Eclipse Adoptium\jdk-17\bin\javaw.exe");
        }

        foreach (var c in candidates)
        {
            if (File.Exists(c)) return c;
        }

        return defaultExe;
    }

    public static int DetectJavaMajorVersion(string javaExe)
    {
        try
        {
            var psi = new ProcessStartInfo
            {
                FileName = javaExe,
                Arguments = "-version",
                UseShellExecute = false,
                RedirectStandardError = true,
                CreateNoWindow = true
            };
            using var proc = Process.Start(psi);
            if (proc != null)
            {
                string output = proc.StandardError.ReadToEnd();
                proc.WaitForExit(3000);

                var match = System.Text.RegularExpressions.Regex.Match(output, @"version\s+""(\d+)(?:\.(\d+))?");
                if (match.Success)
                {
                    int first = int.Parse(match.Groups[1].Value);
                    if (first == 1 && match.Groups[2].Success)
                    {
                        return int.Parse(match.Groups[2].Value); // 1.8 -> 8
                    }
                    return first; // 17, 21, etc.
                }
            }
        }
        catch
        {
            // fallback
        }
        return 17;
    }
}

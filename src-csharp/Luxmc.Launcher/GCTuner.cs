using System.Runtime.InteropServices;

namespace Luxmc.Launcher;

public static class GCTuner
{
    public static List<string> GenerateOptimizedJvmFlags(int memoryMb, int minMemoryMb = 1024, int javaMajor = 17)
    {
        var flags = new List<string>();

        int maxMem = Math.Max(1024, memoryMb);
        int minMem = Math.Clamp(minMemoryMb, 512, maxMem);

        flags.Add($"-Xms{minMem}M");
        flags.Add($"-Xmx{maxMem}M");

        if (javaMajor >= 21 && maxMem >= 8192)
        {
            flags.Add("-XX:+UseZGC");
            flags.Add("-XX:+ZGenerational");
        }
        else
        {
            flags.Add("-XX:+UseG1GC");
            flags.Add("-XX:+ParallelRefProcEnabled");
            flags.Add("-XX:MaxGCPauseMillis=200");
            flags.Add("-XX:+UnlockExperimentalVMOptions");
            flags.Add("-XX:+DisableExplicitGC");
            flags.Add("-XX:+AlwaysPreTouch");
            flags.Add("-XX:G1NewSizePercent=30");
            flags.Add("-XX:G1MaxNewSizePercent=40");
            flags.Add("-XX:G1ReservePercent=20");
            flags.Add("-XX:G1HeapWastePercent=5");
            flags.Add("-XX:G1MixedGCCountTarget=4");
            flags.Add("-XX:InitiatingHeapOccupancyPercent=15");
            flags.Add("-XX:G1MixedGCLiveThresholdPercent=90");
            flags.Add("-XX:G1RSetUpdatingPauseTimePercent=5");
            flags.Add("-XX:SurvivorRatio=32");
            flags.Add("-XX:+PerfDisableSharedMem");
            flags.Add("-XX:MaxTenuringThreshold=1");
        }

        flags.Add("-Dfile.encoding=UTF-8");
        flags.Add("-Dsun.stdout.encoding=UTF-8");
        flags.Add("-Dsun.stderr.encoding=UTF-8");

        if (RuntimeInformation.IsOSPlatform(OSPlatform.Linux))
        {
            flags.Add("-Dorg.lwjgl.glfw.checkerror=false");
            
            var waylandDisplay = Environment.GetEnvironmentVariable("WAYLAND_DISPLAY");
            var xdgSession = Environment.GetEnvironmentVariable("XDG_SESSION_TYPE");
            if (!string.IsNullOrEmpty(waylandDisplay) || string.Equals(xdgSession, "wayland", StringComparison.OrdinalIgnoreCase))
            {
                flags.Add("-Dglfw.platform=x11");
            }
        }
        else if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
        {
            flags.Add("-XX:HeapDumpPath=MojangTricksIntelDriversForPerformance_javaw.exe_minecraft.exe.heapdump");
        }

        return flags;
    }
}

using System.Text.Json;

namespace Luxmc.Launcher;

public class Program
{
    public static async Task<int> Main(string[] args)
    {
        if (args.Length == 0)
        {
            Console.WriteLine(JsonSerializer.Serialize(new
            {
                app = "Luxmc.Launcher",
                engine = ".NET 8 Core Engine",
                version = "1.7.5",
                commands = new[] { "launch", "validate", "diagnose", "gc-tune", "version" }
            }));
            return 0;
        }

        string cmd = args[0].ToLowerInvariant();

        try
        {
            switch (cmd)
            {
                case "launch":
                {
                    if (args.Length < 2)
                    {
                        Console.Error.WriteLine("Erro: requisição de lançamento ausente. Uso: Luxmc.Launcher launch <arquivo-ou-json>");
                        return 1;
                    }

                    string input = args[1];
                    string jsonContent = File.Exists(input) ? await File.ReadAllTextAsync(input) : input;
                    var options = new JsonSerializerOptions { PropertyNameCaseInsensitive = true };
                    var request = JsonSerializer.Deserialize<LaunchRequest>(jsonContent, options) 
                                  ?? throw new InvalidOperationException("Falha ao desserializar LaunchRequest.");

                    var result = await MinecraftLauncher.LaunchAsync(request, supervise: true);
                    Console.WriteLine(JsonSerializer.Serialize(result));

                    // Keep CLI supervisor alive until process finishes if running synchronously
                    return result.Success ? 0 : 2;
                }

                case "validate":
                {
                    string instanceDir = args.Length > 1 ? args[1] : Directory.GetCurrentDirectory();
                    var report = ModpackValidator.Validate(instanceDir);
                    Console.WriteLine(JsonSerializer.Serialize(report, new JsonSerializerOptions { WriteIndented = true }));
                    return report.IsValid ? 0 : 1;
                }

                case "diagnose":
                {
                    if (args.Length < 2)
                    {
                        Console.Error.WriteLine("Erro: log para diagnóstico ausente.");
                        return 1;
                    }
                    string logInput = args[1];
                    string logContent = File.Exists(logInput) ? await File.ReadAllTextAsync(logInput) : logInput;
                    var diagnosis = CrashDoctor.Analyze(logContent);
                    Console.WriteLine(JsonSerializer.Serialize(diagnosis, new JsonSerializerOptions { WriteIndented = true }));
                    return 0;
                }

                case "gc-tune":
                {
                    int ramMb = args.Length > 1 && int.TryParse(args[1], out int parsed) ? parsed : 4096;
                    var flags = GCTuner.GenerateOptimizedJvmFlags(ramMb);
                    Console.WriteLine(JsonSerializer.Serialize(new
                    {
                        ramMb = ramMb,
                        flags = flags,
                        argumentString = string.Join(" ", flags)
                    }, new JsonSerializerOptions { WriteIndented = true }));
                    return 0;
                }

                case "version":
                {
                    Console.WriteLine("Luxmc.Launcher 1.7.5 (.NET 8.0 Native AOT / Single-File)");
                    return 0;
                }

                default:
                {
                    Console.Error.WriteLine($"Comando desconhecido: '{cmd}'");
                    return 1;
                }
            }
        }
        catch (Exception ex)
        {
            var errPayload = new
            {
                success = false,
                error = ex.Message,
                stackTrace = ex.StackTrace
            };
            Console.WriteLine(JsonSerializer.Serialize(errPayload));
            return 99;
        }
    }
}

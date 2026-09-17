using System.IO.Compression;

namespace Luxmc.Launcher;

public static class ModpackValidator
{
    public static ValidationReport Validate(string instanceDir)
    {
        var report = new ValidationReport();

        if (string.IsNullOrWhiteSpace(instanceDir) || !Directory.Exists(instanceDir))
        {
            report.IsValid = false;
            report.Errors.Add($"Diretório da instância não encontrado: {instanceDir}");
            return report;
        }

        string modsDir = Path.Combine(instanceDir, "mods");
        if (!Directory.Exists(modsDir))
        {
            report.LoaderType = "vanilla";
            report.ModCount = 0;
            return report;
        }

        var jarFiles = Directory.GetFiles(modsDir, "*.jar", SearchOption.TopDirectoryOnly);
        report.ModCount = jarFiles.Length;

        bool hasFabric = false;
        bool hasForge = false;
        bool hasNeoForge = false;
        bool hasQuilt = false;

        var seenModNames = new HashSet<string>(StringComparer.OrdinalIgnoreCase);

        foreach (var jar in jarFiles)
        {
            var fileInfo = new FileInfo(jar);
            if (fileInfo.Length < 300)
            {
                report.CorruptedJars.Add(fileInfo.Name);
                report.Errors.Add($"Arquivo de mod corrompido ou incompleto (tamanho {fileInfo.Length} bytes): {fileInfo.Name}");
                report.IsValid = false;
                continue;
            }

            try
            {
                using var archive = ZipFile.OpenRead(jar);
                if (archive.GetEntry("fabric.mod.json") != null) hasFabric = true;
                if (archive.GetEntry("quilt.mod.json") != null) hasQuilt = true;
                if (archive.GetEntry("META-INF/neoforge.mods.toml") != null) hasNeoForge = true;
                if (archive.GetEntry("META-INF/mods.toml") != null) hasForge = true;

                // Base name deduplication heuristic
                string cleanName = fileInfo.Name;
                int dashIdx = cleanName.IndexOf('-');
                if (dashIdx > 2)
                {
                    string baseName = cleanName.Substring(0, dashIdx);
                    if (!seenModNames.Add(baseName) && !baseName.Equals("fabric", StringComparison.OrdinalIgnoreCase))
                    {
                        report.Warnings.Add($"Possível versão duplicada do mod '{baseName}': {fileInfo.Name}");
                    }
                }
            }
            catch
            {
                report.CorruptedJars.Add(fileInfo.Name);
                report.Errors.Add($"Arquivo .jar inválido ou ilegível (não é um arquivo zip válido): {fileInfo.Name}");
                report.IsValid = false;
            }
        }

        if (hasNeoForge) report.LoaderType = "neoforge";
        else if (hasQuilt) report.LoaderType = "quilt";
        else if (hasFabric) report.LoaderType = "fabric";
        else if (hasForge) report.LoaderType = "forge";
        else report.LoaderType = report.ModCount > 0 ? "modded" : "vanilla";

        return report;
    }
}

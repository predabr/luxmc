using System.Text.RegularExpressions;

namespace Luxmc.Launcher;

public static class CrashDoctor
{
    public static DiagnosticResult Analyze(string logContent)
    {
        if (string.IsNullOrWhiteSpace(logContent))
        {
            return new DiagnosticResult
            {
                HasCrash = false,
                Category = "None",
                Title = "Nenhum erro registrado",
                Summary = "O jogo encerrou normalmente sem mensagens de falha.",
                RecommendedFix = string.Empty
            };
        }

        // 1. Out of memory
        if (logContent.Contains("OutOfMemoryError", StringComparison.OrdinalIgnoreCase) ||
            logContent.Contains("There is insufficient memory for the Java Runtime Environment", StringComparison.OrdinalIgnoreCase))
        {
            return new DiagnosticResult
            {
                HasCrash = true,
                Category = "OutOfMemory",
                Title = "Memória RAM Insuficiente",
                Summary = "O Minecraft esgotou toda a memória RAM alocada para o processo Java durante a execução.",
                RecommendedFix = "Abra as configurações da instância ou do Luxmc e aumente a memória RAM alocada para pelo menos 4096 MB ou 6144 MB.",
                RawSnippet = ExtractSnippet(logContent, "OutOfMemoryError", 6)
            };
        }

        // 2. Linux Wayland GLFW error
        if (logContent.Contains("GLFW error 65548", StringComparison.OrdinalIgnoreCase) ||
            (logContent.Contains("Wayland", StringComparison.OrdinalIgnoreCase) && logContent.Contains("window icon", StringComparison.OrdinalIgnoreCase)))
        {
            return new DiagnosticResult
            {
                HasCrash = true,
                Category = "WaylandGlfw",
                Title = "Incompatibilidade GLFW no Wayland",
                Summary = "O GLFW do Minecraft tentou definir o ícone da janela nativa diretamente no Wayland, o que causou o fechamento imediato.",
                RecommendedFix = "O Luxmc ativou a camada de compatibilidade XWayland (-Dglfw.platform=x11) para evitar que o GLFW trave.",
                RawSnippet = ExtractSnippet(logContent, "GLFW error", 6)
            };
        }

        // 3. Windows WGL / OpenGL Driver error
        if (logContent.Contains("WGL: The driver does not appear to support OpenGL", StringComparison.OrdinalIgnoreCase) ||
            logContent.Contains("Failed to create a window", StringComparison.OrdinalIgnoreCase) && logContent.Contains("OpenGL", StringComparison.OrdinalIgnoreCase) ||
            logContent.Contains("Pixel format not accelerated", StringComparison.OrdinalIgnoreCase))
        {
            return new DiagnosticResult
            {
                HasCrash = true,
                Category = "WindowsOpenGL",
                Title = "Driver de Vídeo sem Suporte a OpenGL",
                Summary = "O driver da sua placa de vídeo (GPU) não possui aceleração de hardware OpenGL instalada ou está usando o driver genérico do Windows.",
                RecommendedFix = "Atualize os drivers da sua placa de vídeo (NVIDIA, AMD ou Intel) pelo site oficial do fabricante.",
                RawSnippet = ExtractSnippet(logContent, "OpenGL", 6)
            };
        }

        // 4. Java Version Mismatch
        if (logContent.Contains("UnsupportedClassVersionError", StringComparison.OrdinalIgnoreCase) ||
            logContent.Contains("has been compiled by a more recent version of the Java Runtime", StringComparison.OrdinalIgnoreCase))
        {
            var match = Regex.Match(logContent, @"class file version (\d+\.\d+)");
            string requiredVer = match.Success ? match.Groups[1].Value : "mais recente";
            return new DiagnosticResult
            {
                HasCrash = true,
                Category = "JavaMismatch",
                Title = "Versão do Java Incompatível",
                Summary = $"A versão do Minecraft ou algum dos mods foi compilada para uma versão de Java diferente da selecionada ({requiredVer}).",
                RecommendedFix = "Altere o runtime do Java nas configurações da instância: use Java 21 para Minecraft 1.20.5+, Java 17 para 1.18 a 1.20.4, ou Java 8 para 1.12.2 e versões antigas.",
                RawSnippet = ExtractSnippet(logContent, "UnsupportedClassVersionError", 6)
            };
        }

        // 5. Incompatible Mods / Fabric Loader conflict
        var fabricConflict = Regex.Match(logContent, @"Incompatible mods found![\s\S]*?-\s+Mod\s+['""]?([^'""\n\r]+)['""]?\s+requires\s+['""]?([^'""\n\r]+)['""]?");
        if (fabricConflict.Success)
        {
            string mod = fabricConflict.Groups[1].Value.Trim();
            string req = fabricConflict.Groups[2].Value.Trim();
            return new DiagnosticResult
            {
                HasCrash = true,
                Category = "IncompatibleMod",
                Title = "Conflito de Dependência de Mod",
                Summary = $"O mod '{mod}' requer '{req}', que não está instalado ou está em versão incompatível.",
                RecommendedFix = $"Instale o mod dependente '{req}' através da aba Mods do Luxmc.",
                OffendingMod = mod,
                RawSnippet = fabricConflict.Value
            };
        }

        // 6. Duplicate Mods
        var duplicateMatch = Regex.Match(logContent, @"Duplicate mods? found:[\s\S]*?([a-zA-Z0-9_\-\.]+?\.jar)", RegexOptions.IgnoreCase);
        if (duplicateMatch.Success)
        {
            string modFile = duplicateMatch.Groups[1].Value.Trim();
            return new DiagnosticResult
            {
                HasCrash = true,
                Category = "DuplicateMod",
                Title = "Mods Duplicados Detectados",
                Summary = $"Existem dois arquivos para o mesmo mod instalados na pasta mods: '{modFile}'.",
                RecommendedFix = $"Remova a versão duplicada de '{modFile}' na pasta mods da instância.",
                OffendingMod = modFile,
                RawSnippet = duplicateMatch.Value
            };
        }

        // 7. OptiFine & Fabric / Embeddium conflict
        if (logContent.Contains("OptiFine", StringComparison.OrdinalIgnoreCase) &&
            (logContent.Contains("Sodium", StringComparison.OrdinalIgnoreCase) || logContent.Contains("Embeddium", StringComparison.OrdinalIgnoreCase)))
        {
            return new DiagnosticResult
            {
                HasCrash = true,
                Category = "IncompatibleMod",
                Title = "Incompatibilidade: OptiFine com Sodium/Embeddium",
                Summary = "O OptiFine não é compatível com Sodium ou Embeddium.",
                RecommendedFix = "Desative o OptiFine e use o combo Sodium/Embeddium + Iris/Oculus para ter shaders e máximo de FPS.",
                OffendingMod = "OptiFine",
                RawSnippet = ExtractSnippet(logContent, "OptiFine", 6)
            };
        }

        // 8. Mixin apply failure
        var mixinMatch = Regex.Match(logContent, @"org\.spongepowered\.asm\.mixin\.transformer\.throwables\.MixinTransformerError:.*?critical injection failure.*?in\s+([a-zA-Z0-9_\-\.]+)", RegexOptions.IgnoreCase);
        if (mixinMatch.Success)
        {
            string config = mixinMatch.Groups[1].Value.Trim();
            return new DiagnosticResult
            {
                HasCrash = true,
                Category = "MixinFailure",
                Title = "Falha Crítica de Injeção de Mod (Mixin)",
                Summary = $"Um mod causou erro ao modificar o código do jogo através do mixin '{config}'.",
                RecommendedFix = $"Verifique e atualize os mods associados a '{config}' ou desative-os temporariamente.",
                RawSnippet = mixinMatch.Value
            };
        }

        // 9. Generic Exception match if exit was abnormal
        var exceptionMatch = Regex.Match(logContent, @"(?:Exception in thread|FATAL ERROR|The game crashed whilst|Crash Report)[\s\S]*?(?:at net\.minecraft|at cpw\.mods|at net\.fabricmc)[\s\S]{0,500}", RegexOptions.IgnoreCase);
        if (exceptionMatch.Success)
        {
            return new DiagnosticResult
            {
                HasCrash = true,
                Category = "GenericCrash",
                Title = "Erro na Execução do Minecraft",
                Summary = "Ocorreu uma exceção não tratada durante o carregamento ou execução do jogo.",
                RecommendedFix = "Consulte o relatório de falhas (crash-reports) dentro da pasta da instância para identificar o mod problemático.",
                RawSnippet = exceptionMatch.Value
            };
        }

        return new DiagnosticResult
        {
            HasCrash = false,
            Category = "None",
            Title = "Sem falha crítica detectada",
            Summary = "O processo terminou sem erros conhecidos no log.",
            RecommendedFix = string.Empty
        };
    }

    private static string ExtractSnippet(string text, string pattern, int maxLines)
    {
        int index = text.IndexOf(pattern, StringComparison.OrdinalIgnoreCase);
        if (index == -1) return string.Empty;

        int start = Math.Max(0, index - 200);
        int length = Math.Min(text.Length - start, 800);
        string snippet = text.Substring(start, length);

        var lines = snippet.Split(new[] { '\r', '\n' }, StringSplitOptions.RemoveEmptyEntries);
        return string.Join(Environment.NewLine, lines.Take(maxLines));
    }
}

using System.Text.Json.Serialization;

namespace Luxmc.Launcher;

public class LaunchRequest
{
    [JsonPropertyName("versionId")]
    public string VersionId { get; set; } = string.Empty;

    [JsonPropertyName("gameDir")]
    public string GameDir { get; set; } = string.Empty;

    [JsonPropertyName("assetsDir")]
    public string AssetsDir { get; set; } = string.Empty;

    [JsonPropertyName("assetIndex")]
    public string AssetIndex { get; set; } = string.Empty;

    [JsonPropertyName("javaPath")]
    public string JavaPath { get; set; } = "java";

    [JsonPropertyName("mainClass")]
    public string MainClass { get; set; } = "net.minecraft.client.main.Main";

    [JsonPropertyName("classpath")]
    public List<string> Classpath { get; set; } = new();

    [JsonPropertyName("jvmArgs")]
    public List<string> JvmArgs { get; set; } = new();

    [JsonPropertyName("gameArgs")]
    public List<string> GameArgs { get; set; } = new();

    [JsonPropertyName("username")]
    public string Username { get; set; } = "Player";

    [JsonPropertyName("uuid")]
    public string Uuid { get; set; } = string.Empty;

    [JsonPropertyName("accessToken")]
    public string AccessToken { get; set; } = "-";

    [JsonPropertyName("userType")]
    public string UserType { get; set; } = "mojang";

    [JsonPropertyName("versionType")]
    public string VersionType { get; set; } = "release";

    [JsonPropertyName("serverIp")]
    public string? ServerIp { get; set; }

    [JsonPropertyName("serverPort")]
    public int? ServerPort { get; set; }

    [JsonPropertyName("memoryMb")]
    public int? MemoryMb { get; set; } = 4096;

    [JsonPropertyName("minMemoryMb")]
    public int? MinMemoryMb { get; set; } = 1024;

    [JsonPropertyName("enableVulkan")]
    public bool? EnableVulkan { get; set; }

    [JsonPropertyName("skinUrl")]
    public string? SkinUrl { get; set; }

    [JsonPropertyName("skinVariant")]
    public string? SkinVariant { get; set; }

    [JsonPropertyName("capeUrl")]
    public string? CapeUrl { get; set; }

    [JsonPropertyName("resolutionWidth")]
    public int? ResolutionWidth { get; set; } = 1280;

    [JsonPropertyName("resolutionHeight")]
    public int? ResolutionHeight { get; set; } = 720;

    [JsonPropertyName("isDemo")]
    public bool IsDemo { get; set; } = false;

    [JsonPropertyName("customEnv")]
    public Dictionary<string, string>? CustomEnv { get; set; }
}

public class LaunchResult
{
    [JsonPropertyName("success")]
    public bool Success { get; set; }

    [JsonPropertyName("pid")]
    public int Pid { get; set; }

    [JsonPropertyName("message")]
    public string Message { get; set; } = string.Empty;

    [JsonPropertyName("commandLine")]
    public string CommandLine { get; set; } = string.Empty;
}

public class DiagnosticResult
{
    [JsonPropertyName("hasCrash")]
    public bool HasCrash { get; set; }

    [JsonPropertyName("category")]
    public string Category { get; set; } = "None";

    [JsonPropertyName("title")]
    public string Title { get; set; } = string.Empty;

    [JsonPropertyName("summary")]
    public string Summary { get; set; } = string.Empty;

    [JsonPropertyName("recommendedFix")]
    public string RecommendedFix { get; set; } = string.Empty;

    [JsonPropertyName("offendingMod")]
    public string? OffendingMod { get; set; }

    [JsonPropertyName("rawSnippet")]
    public string? RawSnippet { get; set; }
}

public class ValidationReport
{
    [JsonPropertyName("isValid")]
    public bool IsValid { get; set; } = true;

    [JsonPropertyName("loaderType")]
    public string LoaderType { get; set; } = "vanilla";

    [JsonPropertyName("modCount")]
    public int ModCount { get; set; }

    [JsonPropertyName("corruptedJars")]
    public List<string> CorruptedJars { get; set; } = new();

    [JsonPropertyName("warnings")]
    public List<string> Warnings { get; set; } = new();

    [JsonPropertyName("errors")]
    public List<string> Errors { get; set; } = new();
}

public class JavaRuntimeInfo
{
    [JsonPropertyName("path")]
    public string Path { get; set; } = string.Empty;

    [JsonPropertyName("version")]
    public string Version { get; set; } = string.Empty;

    [JsonPropertyName("majorVersion")]
    public int MajorVersion { get; set; }

    [JsonPropertyName("vendor")]
    public string Vendor { get; set; } = string.Empty;

    [JsonPropertyName("is64Bit")]
    public bool Is64Bit { get; set; } = true;
}

public class StartupHardwareProbe {
    public static void main(String[] args) {
        long started = System.nanoTime();
        oshi.software.os.OSProcess process = new oshi.SystemInfo().getOperatingSystem().getCurrentProcess();
        if (process == null || process.getProcessID() <= 0) throw new AssertionError("Current process not available");
        System.out.println("{\"hardwareSeconds\":" + ((System.nanoTime() - started) / 1000000000.0) + ",\"processAvailable\":true}");
    }
}

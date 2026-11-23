import { useState } from "react";
import { invoke, isTauri } from "@tauri-apps/api/core";

type ConnectionMode = "usb" | "wireless";
type AndroidDevice = {
  serial: string; state: string; model: string | null;
  product: string | null; device: string | null;
};

function App() {
  const [mode, setMode] = useState<ConnectionMode>("usb");
  const [devices, setDevices] = useState<AndroidDevice[]>([]);
  const [selectedSerial, setSelectedSerial] = useState("");
  const [scanning, setScanning] = useState(false);
  const [scanError, setScanError] = useState("");
  const [hasScanned, setHasScanned] = useState(false);
  const [scrcpyRunning, setScrcpyRunning] = useState(false);
  const [scrcpyBusy, setScrcpyBusy] = useState(false);
  const [scrcpyMessage, setScrcpyMessage] = useState("");

  async function detectDevices() {
    setScanError("");
    setScrcpyMessage("");
    setScanning(true);
    setHasScanned(false);
    try {
      if (!isTauri()) throw new Error("USB scanning is available in the MirrorX Windows desktop app, not in this website.");
      const found = await invoke<AndroidDevice[]>("list_android_devices");
      setDevices(found);
      setSelectedSerial(found.find((device) => device.state === "device")?.serial ?? "");
      setHasScanned(true);
      const running = await invoke<boolean>("scrcpy_status");
      setScrcpyRunning(running);
    } catch (error) {
      setDevices([]);
      setSelectedSerial("");
      setScanError(error instanceof Error ? error.message : String(error));
    } finally {
      setScanning(false);
    }
  }

  async function startMirroring() {
    if (!selectedSerial) return;
    setScrcpyBusy(true);
    setScrcpyMessage("");
    try {
      if (!isTauri()) throw new Error("Mirroring can only start in the MirrorX Windows desktop app.");
      const message = await invoke<string>("start_scrcpy", { serial: selectedSerial });
      setScrcpyMessage(message);
      setScrcpyRunning(true);
    } catch (error) {
      setScrcpyMessage(error instanceof Error ? error.message : String(error));
    } finally {
      setScrcpyBusy(false);
    }
  }

  async function stopMirroring() {
    setScrcpyBusy(true);
    setScrcpyMessage("");
    try {
      if (!isTauri()) throw new Error("Mirroring can only be controlled in the MirrorX Windows desktop app.");
      const message = await invoke<string>("stop_scrcpy");
      setScrcpyMessage(message);
      setScrcpyRunning(false);
    } catch (error) {
      setScrcpyMessage(error instanceof Error ? error.message : String(error));
    } finally {
      setScrcpyBusy(false);
    }
  }

  const connectedCount = devices.filter((device) => device.state === "device").length;
  const selectedDevice = devices.find((device) => device.serial === selectedSerial);

  return (
    <main className="app-shell">
      <aside className="sidebar">
        <div className="brand"><div className="brand-mark">M</div><div><strong>MirrorX</strong><span>ANDROID CONNECT</span></div></div>
        <nav className="nav-list" aria-label="Main navigation">
          <div className="nav-item active"><span className="nav-icon">▣</span> Dashboard</div>
          <div className="nav-item muted"><span className="nav-icon">◈</span> Devices</div>
          <div className="nav-item muted"><span className="nav-icon">⚙</span> Settings</div>
        </nav>
        <div className="sidebar-bottom">
          <div className="engine-status"><span className={scrcpyRunning ? "status-dot running" : "status-dot"} /> {scrcpyRunning ? "scrcpy session launched" : "ADB / scrcpy prototype"}</div>
          <span className="version">MIRRORX · EARLY ACCESS</span>
        </div>
      </aside>

      <section className="workspace">
        <header className="topbar">
          <div><span className="eyebrow">WORKSPACE / OVERVIEW</span><h1>Dashboard</h1></div>
          <div className="topbar-right"><span className="avatar">MX</span></div>
        </header>

        <section className="welcome">
          <div>
            <span className="eyebrow">YOUR PHONE. YOUR PC. ONE EXPERIENCE.</span>
            <h2>Connect your Android<br /><em>to a bigger workspace.</em></h2>
            <p>Discover Android devices and launch the scrcpy mirroring window from your Windows desktop.</p>
          </div>
          <div className="device-art" aria-hidden="true">
            <div className="orbit orbit-one" /><div className="orbit orbit-two" />
            <div className="phone"><div className="phone-camera" /><div className="phone-screen">
              <div className="screen-glow" /><div className="screen-line line-one" />
              <div className="screen-line line-two" /><div className="screen-line line-three" />
            </div></div>
          </div>
        </section>

        <section className="section-heading">
          <div><span className="eyebrow">CONNECTION</span><h3>Connect a device</h3></div>
          <span className="offline-label"><span className="status-dot" /> {connectedCount > 0 ? `${connectedCount} DEVICE${connectedCount === 1 ? "" : "S"} READY` : "NOT CONNECTED"}</span>
        </section>

        <section className="connect-card">
          <div className="connect-tabs" role="tablist" aria-label="Connection type">
            <button className={mode === "usb" ? "tab selected" : "tab"} onClick={() => setMode("usb")} role="tab" aria-selected={mode === "usb"}>USB cable</button>
            <button className={mode === "wireless" ? "tab selected" : "tab"} onClick={() => setMode("wireless")} role="tab" aria-selected={mode === "wireless"}>Wireless</button>
          </div>
          {mode === "usb" ? (
            <div className="connect-content">
              <div className="connect-symbol">⌁</div>
              <div className="connect-copy">
                <h4>Find Android devices over USB</h4>
                <p>Connect your phone with a data-capable USB cable. Enable USB debugging in Android Developer options and approve the authorization prompt on your phone.</p>
                <ol>
                  <li>Connect your phone to the Windows computer.</li>
                  <li>Enable USB debugging on your Android device.</li>
                  <li>Unlock your phone and approve the computer authorization prompt.</li>
                </ol>
                <button className="primary-button" onClick={detectDevices} disabled={scanning}>{scanning ? "Scanning…" : "Detect devices"} <span>→</span></button>
                {scanError && <div className="scan-message error" role="alert">{scanError}</div>}
                {hasScanned && devices.length === 0 && <div className="scan-message" role="status">No devices found. Check the USB cable, USB debugging setting, and Android authorization prompt, then scan again.</div>}
                {devices.length > 0 && <div className="device-list" aria-live="polite">
                  <div className="device-list-heading">DETECTED DEVICES · SELECT ONE</div>
                  {devices.map((device) => <button className={selectedSerial === device.serial ? "device-row selected" : "device-row"} key={device.serial} onClick={() => setSelectedSerial(device.serial)} type="button">
                    <span className="device-info"><strong>{device.model || device.product || "Android device"}</strong><span>{device.serial}</span></span>
                    <span className={device.state === "device" ? "device-state ready" : "device-state"}>{device.state}</span>
                  </button>)}
                </div>}
                {selectedDevice?.state === "device" && <div className="mirror-panel">
                  <div className="mirror-heading"><div><span className="eyebrow">SELECTED DEVICE</span><h4>{selectedDevice.model || selectedDevice.product || "Android device"}</h4></div><span className={scrcpyRunning ? "session-pill active" : "session-pill"}>{scrcpyRunning ? "SESSION LAUNCHED" : "READY TO MIRROR"}</span></div>
                  <p>MirrorX will launch scrcpy in its own desktop window. Install scrcpy and make sure it is available in PATH.</p>
                  <div className="mirror-actions">
                    <button className="primary-button" onClick={startMirroring} disabled={scrcpyBusy || scrcpyRunning}>{scrcpyBusy ? "Please wait…" : "Start mirroring"} <span>↗</span></button>
                    <button className="secondary-button" onClick={stopMirroring} disabled={scrcpyBusy || !scrcpyRunning}>Stop session</button>
                  </div>
                  {scrcpyMessage && <div className="scan-message" role="status">{scrcpyMessage}</div>}
                </div>}
                {selectedDevice && selectedDevice.state !== "device" && <div className="scan-message error">This device is not authorized yet. Unlock your phone and approve the USB debugging prompt.</div>}
                <p className="device-note">Mirroring runs locally. The website cannot access USB devices; use the Windows desktop app.</p>
              </div>
            </div>
          ) : (
            <div className="connect-content"><div className="connect-symbol">⌁</div><div className="connect-copy">
              <h4>Wireless connection is planned</h4>
              <p>Wireless debugging will be added after USB connection is working reliably. It will require an explicit setup and approval on your Android device.</p>
              <span className="planned-tag">PLANNED FEATURE</span>
            </div></div>
          )}
        </section>

        <section className="platform-section">
          <div className="section-heading platform-heading">
            <div><span className="eyebrow">ONE PRODUCT · MULTIPLE PLATFORMS</span><h3>MirrorX everywhere</h3></div>
            <span className="planned-tag">IN DEVELOPMENT</span>
          </div>
          <div className="platform-grid">
            <article className="platform-card"><span className="platform-icon">▣</span><div><h4>Windows</h4><p>Desktop host for Android mirroring and remote control.</p><span className="platform-status">DESKTOP PROTOTYPE</span></div></article>
            <article className="platform-card"><span className="platform-icon">◉</span><div><h4>Android</h4><p>Send your phone screen to PC and use mobile remote controls.</p><span className="platform-status">MOBILE APP FOUNDATION</span></div></article>
            <article className="platform-card"><span className="platform-icon">◌</span><div><h4>iOS</h4><p>Screen broadcasting and remote-control companion features.</p><span className="platform-status">MOBILE APP FOUNDATION</span></div></article>
          </div>
          <p className="platform-note">Mobile apps are in development. Downloads are not available yet. Screen sharing and remote control will require user approval and supported platform permissions.</p>
        </section>

        <footer className="footer"><span>MirrorX is under active development.</span><span>Local-first · Privacy-minded · Windows first</span></footer>
      </section>
    </main>
  );
}

export default App;

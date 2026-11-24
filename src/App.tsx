import { useState } from "react";
import MarketingSite from "./MarketingSite";
import { invoke, isTauri } from "@tauri-apps/api/core";

type ConnectionMode = "usb" | "wireless";
type AndroidDevice = {
  serial: string; state: string; model: string | null;
  product: string | null; device: string | null;
};

function App() {
  if (!isTauri()) return <MarketingSite />;
  const [mode, setMode] = useState<ConnectionMode>("usb");
  const [devices, setDevices] = useState<AndroidDevice[]>([]);
  const [selectedSerial, setSelectedSerial] = useState("");
  const [scanning, setScanning] = useState(false);
  const [scanError, setScanError] = useState("");
  const [hasScanned, setHasScanned] = useState(false);
  const [scrcpyRunning, setScrcpyRunning] = useState(false);
  const [scrcpyBusy, setScrcpyBusy] = useState(false);
  const [scrcpyMessage, setScrcpyMessage] = useState("");
  const [wirelessHost, setWirelessHost] = useState("");
  const [pairingPort, setPairingPort] = useState("");
  const [connectionPort, setConnectionPort] = useState("");
  const [pairingCode, setPairingCode] = useState("");
  const [wirelessBusy, setWirelessBusy] = useState(false);
  const [wirelessMessage, setWirelessMessage] = useState("");

  async function detectDevices() {
    setScanError(""); setScrcpyMessage(""); setScanning(true); setHasScanned(false);
    try {
      if (!isTauri()) throw new Error("USB scanning is available in the MirrorX Windows desktop app, not in this website.");
      const found = await invoke<AndroidDevice[]>("list_android_devices");
      setDevices(found);
      setSelectedSerial(found.find((device) => device.state === "device")?.serial ?? "");
      setHasScanned(true);
      setScrcpyRunning(await invoke<boolean>("scrcpy_status"));
    } catch (error) {
      setDevices([]); setSelectedSerial("");
      setScanError(error instanceof Error ? error.message : String(error));
    } finally { setScanning(false); }
  }

  async function pairWirelessDevice() {
    setWirelessBusy(true); setWirelessMessage("");
    try {
      if (!isTauri()) throw new Error("Wireless pairing is available in the MirrorX Windows desktop app.");
      const message = await invoke<string>("pair_android_wireless", {
        host: wirelessHost.trim(), port: Number(pairingPort), pairingCode: pairingCode.trim(),
      });
      setWirelessMessage(message); setPairingCode("");
    } catch (error) {
      setWirelessMessage(error instanceof Error ? error.message : String(error));
    } finally { setWirelessBusy(false); }
  }

  async function connectWirelessDevice() {
    setWirelessBusy(true); setWirelessMessage("");
    try {
      if (!isTauri()) throw new Error("Wireless connection is available in the MirrorX Windows desktop app.");
      const message = await invoke<string>("connect_android_wireless", {
        host: wirelessHost.trim(), port: Number(connectionPort),
      });
      setWirelessMessage(message);
      const found = await invoke<AndroidDevice[]>("list_android_devices");
      setDevices(found);
      setSelectedSerial(found.find((device) => device.state === "device")?.serial ?? "");
      setHasScanned(true);
    } catch (error) {
      setWirelessMessage(error instanceof Error ? error.message : String(error));
    } finally { setWirelessBusy(false); }
  }

  async function startMirroring() {
    if (!selectedSerial) return;
    setScrcpyBusy(true); setScrcpyMessage("");
    try {
      if (!isTauri()) throw new Error("Mirroring can only start in the MirrorX Windows desktop app.");
      const message = await invoke<string>("start_scrcpy", { serial: selectedSerial });
      setScrcpyMessage(message); setScrcpyRunning(true);
    } catch (error) {
      setScrcpyMessage(error instanceof Error ? error.message : String(error));
    } finally { setScrcpyBusy(false); }
  }

  async function stopMirroring() {
    setScrcpyBusy(true); setScrcpyMessage("");
    try {
      if (!isTauri()) throw new Error("Mirroring can only be controlled in the MirrorX Windows desktop app.");
      const message = await invoke<string>("stop_scrcpy");
      setScrcpyMessage(message); setScrcpyRunning(false);
    } catch (error) {
      setScrcpyMessage(error instanceof Error ? error.message : String(error));
    } finally { setScrcpyBusy(false); }
  }

  const connectedCount = devices.filter((device) => device.state === "device").length;
  const selectedDevice = devices.find((device) => device.serial === selectedSerial);

  return (
    <main className="mx-app">
      <header className="mx-header">
        <a className="mx-brand" href="/" aria-label="MirrorX home">
          <span className="mx-brand-symbol">M<span>.</span></span>
          <span className="mx-brand-name">mirror<span>x</span><small>DEVICE EXPERIENCE PLATFORM</small></span>
        </a>
        <div className="mx-header-center"><span className="header-live-dot" /> PRIVATE WORKSPACE <i /> WINDOWS DESKTOP</div>
        <div className="mx-header-right"><span className="header-status">{scrcpyRunning ? "SESSION ACTIVE" : "READY WHEN YOU ARE"}</span><span className="mx-avatar">MX</span></div>
      </header>

      <div className="mx-main">
        <section className="mx-intro">
          <div className="mx-breadcrumb">WORKSPACE <span>/</span> DEVICE STUDIO</div>
          <div className="mx-intro-row">
            <div><p className="mx-kicker">THE SPACE BETWEEN DEVICES, GONE.</p><h1>Your devices.<br/><em>In perfect sync.</em></h1>
              <p className="mx-lede">A focused workspace for bringing your Android screen to your Windows desktop. Connect locally, stay in control.</p>
            </div>
            <div className="mx-intro-meta"><span className="meta-label">CURRENT WORKSPACE</span><strong>Personal setup</strong><span className="meta-divider"/><span className="meta-label">CONNECTION</span><strong>{connectedCount ? `${connectedCount} device ready` : "No device connected"}</strong></div>
          </div>
        </section>

        <section className="mx-hero">
          <div className="mx-hero-copy">
            <span className="mx-overline"><span/> MIRRORX DEVICE STUDIO</span>
            <h2>Your phone,<br/>without the limits<br/><em>of its screen.</em></h2>
            <p>Move from pocket to desktop. Keep your workflow in one place with a direct, local connection.</p>
            <a className="mx-text-link" href="#connect">Set up your connection <span>↘</span></a>
          </div>
          <div className="mx-visual" aria-hidden="true">
            <div className="visual-orbit orbit-a"/><div className="visual-orbit orbit-b"/><div className="visual-glow"/>
            <div className="visual-device visual-device-back"><div className="visual-device-screen"><span/><span/><span/></div></div>
            <div className="visual-device visual-device-front"><div className="visual-camera"/><div className="visual-device-screen"><div className="visual-screen-top">MIRRORX <b>•••</b></div><div className="visual-screen-card"><small>YOUR DEVICE</small><strong>Connected<br/>to your space.</strong><i/></div><div className="visual-screen-bars"><span/><span/><span/></div></div></div>
            <div className="visual-chip chip-one"><span className="chip-icon">↗</span><span><b>Direct connection</b><small>Local device link</small></span></div>
            <div className="visual-chip chip-two"><span className="chip-check">✓</span><span><b>Your control</b><small>Permission first</small></span></div>
          </div>
          <div className="hero-index"><span>01</span><i/> DEVICE CONNECTION</div>
        </section>

        <section className="mx-connect-section" id="connect">
          <div className="mx-section-head"><div><p className="mx-kicker">START WITH A CONNECTION</p><h2>Bring a device online.</h2><p>Choose how you want to connect your Android phone.</p></div><div className="mx-connection-state"><span className={connectedCount ? "state-light connected" : "state-light"}/><span>{connectedCount ? `${connectedCount} DEVICE READY` : "WAITING FOR DEVICE"}</span></div></div>
          <div className="mx-connect-layout">
            <section className="mx-connect-card">
              <div className="mx-card-top"><div><span className="mx-card-number">01 / CONNECTION METHOD</span><h3>Connect your Android</h3></div><span className="mx-card-mark">MX</span></div>
              <div className="mx-method-switch" role="tablist" aria-label="Connection type">
                <button className={mode === "usb" ? "method-tab active" : "method-tab"} onClick={() => setMode("usb")} role="tab" aria-selected={mode === "usb"}><span>⌁</span> USB cable <small>RECOMMENDED</small></button>
                <button className={mode === "wireless" ? "method-tab active" : "method-tab"} onClick={() => setMode("wireless")} role="tab" aria-selected={mode === "wireless"}><span>⌁</span> Wireless</button>
              </div>
              {mode === "usb" ? (
                <div className="mx-form-content">
                  <div className="mx-instruction"><span className="instruction-icon">01</span><div><strong>Connect your phone</strong><p>Use a data-capable USB cable and connect your Android device to your Windows PC.</p></div></div>
                  <div className="mx-instruction"><span className="instruction-icon">02</span><div><strong>Allow USB debugging</strong><p>Enable Developer options → USB debugging, then approve the computer on your phone.</p></div></div>
                  <button className="mx-primary-button" onClick={detectDevices} disabled={scanning}>{scanning ? "Scanning devices…" : "Detect connected devices"} <span>↗</span></button>
                  {scanError && <div className="scan-message error" role="alert">{scanError}</div>}
                  {hasScanned && devices.length === 0 && <div className="scan-message" role="status">No devices found. Check your cable, USB debugging and phone authorization, then try again.</div>}
                  {devices.length > 0 && <div className="device-list" aria-live="polite"><div className="device-list-heading">DETECTED DEVICES · SELECT ONE</div>
                    {devices.map((device) => <button className={selectedSerial === device.serial ? "device-row selected" : "device-row"} key={device.serial} onClick={() => setSelectedSerial(device.serial)} type="button">
                      <span className="device-info"><strong>{device.model || device.product || "Android device"}</strong><span>{device.serial}</span></span>
                      <span className={device.state === "device" ? "device-state ready" : "device-state"}>{device.state}</span>
                    </button>)}</div>}
                  {selectedDevice?.state === "device" && <div className="mirror-panel">
                    <div className="mirror-heading"><div><span className="eyebrow">SELECTED DEVICE</span><h4>{selectedDevice.model || selectedDevice.product || "Android device"}</h4></div><span className={scrcpyRunning ? "session-pill active" : "session-pill"}>{scrcpyRunning ? "SESSION LAUNCHED" : "READY TO MIRROR"}</span></div>
                    <p>MirrorX launches scrcpy in a separate desktop window. Install scrcpy and ensure it is available in PATH.</p>
                    <div className="mirror-actions"><button className="mx-primary-button" onClick={startMirroring} disabled={scrcpyBusy || scrcpyRunning}>{scrcpyBusy ? "Please wait…" : "Start mirroring"} <span>↗</span></button><button className="secondary-button" onClick={stopMirroring} disabled={scrcpyBusy || !scrcpyRunning}>Stop session</button></div>
                    {scrcpyMessage && <div className="scan-message" role="status">{scrcpyMessage}</div>}
                  </div>}
                  {selectedDevice && selectedDevice.state !== "device" && <div className="scan-message error">This device is not authorized yet. Unlock your phone and approve the USB debugging prompt.</div>}
                  <p className="mx-privacy-note"><span>◈</span> Device discovery is performed by the Windows desktop app. This website cannot access USB hardware.</p>
                </div>
              ) : (
                <div className="mx-form-content">
                  <p className="mx-wireless-intro">Open Developer options → Wireless debugging → Pair device with pairing code. Keep both devices on the same network.</p>
                  <div className="wireless-form">
                    <label>PHONE IP ADDRESS<input value={wirelessHost} onChange={(event) => setWirelessHost(event.target.value)} placeholder="192.168.1.25" autoComplete="off"/></label>
                    <div className="wireless-fields"><label>PAIRING PORT<input value={pairingPort} onChange={(event) => setPairingPort(event.target.value)} placeholder="Pairing port" inputMode="numeric"/></label><label>6-DIGIT PAIRING CODE<input value={pairingCode} onChange={(event) => setPairingCode(event.target.value)} placeholder="123456" inputMode="numeric" maxLength={6}/></label></div>
                    <button className="mx-primary-button" onClick={pairWirelessDevice} disabled={wirelessBusy || !wirelessHost.trim() || !pairingPort || pairingCode.length !== 6}>{wirelessBusy ? "Please wait…" : "Pair this device"} <span>↗</span></button>
                    <label>CONNECTION PORT<input value={connectionPort} onChange={(event) => setConnectionPort(event.target.value)} placeholder="Connection port shown separately" inputMode="numeric"/></label>
                    <button className="secondary-button" onClick={connectWirelessDevice} disabled={wirelessBusy || !wirelessHost.trim() || !connectionPort}>{wirelessBusy ? "Please wait…" : "Connect over Wi-Fi"}</button>
                    {wirelessMessage && <div className="scan-message" role="status">{wirelessMessage}</div>}
                    <p className="device-note">Android provides separate pairing and connection ports. Enter the exact values shown in Wireless debugging. Pairing uses ADB's authenticated flow.</p>
                  </div>
                </div>
              )}
            </section>

            <aside className="mx-side-panel">
              <div className="side-panel-head"><span className="mx-card-number">02 / HOW IT WORKS</span><span className="side-panel-icon">↗</span></div>
              <h3>A simple bridge.<br/><em>Nothing in the way.</em></h3>
              <div className="flow-step"><span>01</span><div><b>Connect</b><p>Pair your Android device over USB or your local Wi-Fi network.</p></div></div>
              <div className="flow-line"/>
              <div className="flow-step"><span>02</span><div><b>Approve</b><p>Your device stays in control. Approve debugging and capture permissions.</p></div></div>
              <div className="flow-line"/>
              <div className="flow-step"><span>03</span><div><b>Work bigger</b><p>Launch the desktop mirroring window from the Windows app.</p></div></div>
              <div className="side-panel-foot"><span>◈</span> LOCAL-FIRST BY DESIGN</div>
            </aside>
          </div>
        </section>

        <section className="mx-platform-section">
          <div className="mx-section-head"><div><p className="mx-kicker">ONE ECOSYSTEM, MULTIPLE SCREENS</p><h2>Built to move with you.</h2><p>MirrorX is being developed across desktop and mobile platforms.</p></div><span className="mx-roadmap-label">PRODUCT ROADMAP</span></div>
          <div className="mx-platform-grid">
            <article className="mx-platform-card"><div className="platform-card-top"><span className="platform-icon">▣</span><span className="platform-number">01</span></div><h3>Windows</h3><p>Desktop host for Android device discovery and scrcpy-based mirroring.</p><span className="platform-status">DESKTOP PROTOTYPE</span></article>
            <article className="mx-platform-card"><div className="platform-card-top"><span className="platform-icon">◉</span><span className="platform-number">02</span></div><h3>Android</h3><p>Mobile capture and companion controls are under active development.</p><span className="platform-status">FOUNDATION IN PROGRESS</span></article>
            <article className="mx-platform-card"><div className="platform-card-top"><span className="platform-icon">◌</span><span className="platform-number">03</span></div><h3>iOS</h3><p>ReplayKit-based broadcasting and companion features are planned.</p><span className="platform-status">FOUNDATION IN PROGRESS</span></article>
          </div>
          <p className="mx-roadmap-note">Mobile downloads are not available yet. iOS capture, cross-platform streaming and remote control are not represented as finished features.</p>
        </section>
      </div>
      <footer className="mx-footer"><a className="mx-footer-brand" href="/">M<span>.</span> MIRRORX</a><span>Designed for a more connected workspace.</span><span>LOCAL-FIRST <i/> PRIVACY-MINDED <i/> IN DEVELOPMENT</span></footer>
    </main>
  );
}
export default App;

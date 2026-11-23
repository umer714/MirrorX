import { useState } from "react";

type ConnectionMode = "usb" | "wireless";

function App() {
  const [mode, setMode] = useState<ConnectionMode>("usb");

  return (
    <main className="app-shell">
      <aside className="sidebar">
        <div className="brand">
          <div className="brand-mark">M</div>
          <div>
            <strong>MirrorX</strong>
            <span>ANDROID CONNECT</span>
          </div>
        </div>

        <nav className="nav-list" aria-label="Main navigation">
          <div className="nav-item active"><span className="nav-icon">▣</span> Dashboard</div>
          <div className="nav-item muted"><span className="nav-icon">◈</span> Devices</div>
          <div className="nav-item muted"><span className="nav-icon">⚙</span> Settings</div>
        </nav>

        <div className="sidebar-bottom">
          <div className="engine-status"><span className="status-dot" /> Engine setup pending</div>
          <span className="version">MIRRORX · EARLY ACCESS</span>
        </div>
      </aside>

      <section className="workspace">
        <header className="topbar">
          <div>
            <span className="eyebrow">WORKSPACE / OVERVIEW</span>
            <h1>Dashboard</h1>
          </div>
          <div className="topbar-right"><span className="avatar">MX</span></div>
        </header>

        <section className="welcome">
          <div>
            <span className="eyebrow">YOUR PHONE. YOUR PC. ONE EXPERIENCE.</span>
            <h2>Connect your Android<br /><em>to a bigger workspace.</em></h2>
            <p>Mirror your screen and interact from your desktop. Connect a device to get started.</p>
          </div>
          <div className="device-art" aria-hidden="true">
            <div className="orbit orbit-one" />
            <div className="orbit orbit-two" />
            <div className="phone">
              <div className="phone-camera" />
              <div className="phone-screen">
                <div className="screen-glow" />
                <div className="screen-line line-one" />
                <div className="screen-line line-two" />
                <div className="screen-line line-three" />
              </div>
            </div>
          </div>
        </section>

        <section className="section-heading">
          <div>
            <span className="eyebrow">CONNECTION</span>
            <h3>Connect a device</h3>
          </div>
          <span className="offline-label"><span className="status-dot" /> NOT CONNECTED</span>
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
                <h4>Connect with a USB cable</h4>
                <p>Connect your Android phone to this computer. Make sure USB debugging is enabled and approve the authorization prompt on your phone.</p>
                <ol>
                  <li>Connect your phone using a data-capable USB cable.</li>
                  <li>Enable Developer options and USB debugging on Android.</li>
                  <li>Approve this computer when Android asks for permission.</li>
                </ol>
                <button className="primary-button" disabled title="Device detection will be available in a future build">Detect devices <span>→</span></button>
              </div>
            </div>
          ) : (
            <div className="connect-content">
              <div className="connect-symbol">⌁</div>
              <div className="connect-copy">
                <h4>Wireless connection is planned</h4>
                <p>Wireless debugging will be added after USB connection is working reliably. It will require an explicit setup and approval on your Android device.</p>
                <span className="planned-tag">PLANNED FEATURE</span>
              </div>
            </div>
          )}
        </section>

        <footer className="footer">
          <span>MirrorX is under active development.</span>
          <span>Local-first · Privacy-minded · Windows first</span>
        </footer>
      </section>
    </main>
  );
}

export default App;
import "./marketing.css";

const features = [
  { n: "01", icon: "▣", title: "Your screen, in focus", body: "Bring your Android display into your Windows workspace with a clean, purpose-built viewing experience.", label: "SCREEN EXPERIENCE" },
  { n: "02", icon: "⌁", title: "Choose how you connect", body: "USB and trusted local wireless workflows, with clear setup steps and no confusing cloud-first flow.", label: "FLEXIBLE CONNECTION" },
  { n: "03", icon: "◈", title: "Control stays with you", body: "Visible capture permission, deliberate pairing and clear session controls are central to the product.", label: "PRIVACY BY DESIGN" },
];

export default function MarketingSite() {
  return <main className="mx-site">
    <header className="site-nav">
      <a className="site-logo" href="#home" aria-label="MirrorX home"><span className="site-logo-mark">M<span>.</span></span><span>mirror<span>x</span></span></a>
      <nav aria-label="Main navigation"><a href="#experience">Product</a><a href="#features">Experience</a><a href="#security">Security</a><a href="#platforms">Platforms</a></nav>
      <a className="site-nav-cta" href="#get-started">Explore MirrorX <span>↗</span></a>
    </header>

    <section className="site-hero" id="home">
      <div className="site-hero-grid"/>
      <div className="site-hero-copy">
        <div className="site-eyebrow"><i/> THE SPACE BETWEEN DEVICES, REIMAGINED</div>
        <h1>Your phone.<br/><em>Without limits.</em></h1>
        <p className="site-hero-lede">A more considered way to bring your Android screen to your Windows workspace. One connected experience, designed around clarity, speed and control.</p>
        <div className="site-hero-actions"><a className="site-button-primary" href="#experience">Discover MirrorX <span>↗</span></a><a className="site-button-text" href="#features">Explore the experience <span>↓</span></a></div>
        <div className="site-proof"><span><i/> WINDOWS DESKTOP</span><span><i/> ANDROID CAPTURE FOUNDATION</span><span><i/> LOCAL-FIRST APPROACH</span></div>
      </div>
      <div className="site-art" aria-label="MirrorX device workspace concept">
        <div className="art-backdrop-orbit orbit-one"/><div className="art-backdrop-orbit orbit-two"/>
        <div className="art-glow"/>
        <div className="site-monitor"><div className="monitor-top"><span><b className="monitor-brand-dot"/> MIRRORX <small>/ WORKSPACE</small></span><span className="monitor-window-dots">● ● ●</span></div><div className="monitor-content"><div className="monitor-side"><b>MX</b><i className="selected"/><i/><i/><i/></div><div className="monitor-screen"><div className="monitor-screen-top"><span>DEVICE OVERVIEW</span><span className="monitor-live"><i/> READY</span></div><div className="monitor-screen-center"><small>YOUR PHONE. YOUR PC.</small><strong>One view.<br/><em>More possibility.</em></strong><p>Seamlessly bring your devices together.</p></div><div className="monitor-screen-bottom"><span><i/> LOCAL CONNECTION</span><span>01 / 03</span></div></div></div><div className="monitor-stand"/></div>
        <div className="site-phone"><div className="phone-camera"/><div className="phone-ui"><small>MIRRORX MOBILE</small><div className="phone-wallpaper-orb"/><b>Your world,<br/>in sync.</b><span>DEVICE COMPANION</span><i/><i/><i/></div></div>
        <div className="site-float float-top"><span>⌁</span><div><b>Connected by design</b><small>Local-first direction</small></div></div>
        <div className="site-float float-bottom"><span>◈</span><div><b>Permission matters</b><small>Control stays yours</small></div></div>
      </div>
      <div className="site-hero-foot"><span>01 — THE MIRRORX EXPERIENCE</span><span>SCROLL TO DISCOVER ↓</span></div>
    </section>

    <section className="site-manifesto" id="experience"><div className="site-section-index">A BETTER WAY TO WORK ACROSS SCREENS <span>— 01</span></div><div className="site-manifesto-grid"><h2>Less friction.<br/><em>More focus.</em></h2><div><p>Your phone is part of how you work, play and create. MirrorX is being built to make that connection feel natural on the screen in front of you—without adding another complicated layer.</p><p className="site-muted">A focused desktop companion. Straightforward device workflows. Clear permission at every important step.</p></div></div><div className="manifesto-ribbon"><span>01 / CONNECT</span><i/><span>02 / VIEW</span><i/><span>03 / STAY IN CONTROL</span></div></section>

    <section className="site-capabilities" id="features"><div className="site-section-heading"><div><div className="site-section-index">DESIGNED AROUND THE DETAILS <span>— 02</span></div><h2>Simple on the surface.<br/><em>Thoughtful underneath.</em></h2></div><p>Every part of the experience has a purpose: make connecting devices feel clear, dependable and under your control.</p></div><div className="site-capability-grid">{features.map((item) => <article className="site-capability" key={item.n}><div className="cap-top"><span>{item.n}</span><span className="cap-icon">{item.icon}</span></div><div className="cap-tag">{item.label}</div><h3>{item.title}</h3><p>{item.body}</p><div className="cap-bottom"><span>EXPLORE PRINCIPLE</span><span>↗</span></div></article>)}</div></section>

    <section className="site-security" id="security"><div className="security-orb"/><div className="site-section-index">TRUST IS PART OF THE INTERFACE <span>— 03</span></div><div className="security-content"><div><div className="security-symbol">◈</div><h2>Your devices.<br/><em>Your permission.</em></h2></div><div className="security-copy"><p>Sharing a screen should never feel mysterious. MirrorX is designed around visible capture approval, deliberate pairing and clear session controls.</p><ul><li><span>01</span> Local-first media direction</li><li><span>02</span> Explicit capture approval</li><li><span>03</span> Separate control permission</li><li><span>04</span> Clear session revocation</li></ul><small>These are product design principles. Authenticated streaming and remote control remain under development.</small></div></div></section>

    <section className="site-platforms" id="platforms"><div className="site-section-index">BUILT IN CLEAR STAGES <span>— 04</span></div><div className="platform-heading"><h2>One vision.<br/><em>Across your devices.</em></h2><p>MirrorX is being developed in focused stages. We show platform status honestly—so you can distinguish working foundations from features still in progress.</p></div><div className="platform-list"><article><span>01</span><div><h3>Windows desktop</h3><p>Device discovery and scrcpy-based launch prototype.</p></div><b className="status-prototype">PROTOTYPE</b></article><article><span>02</span><div><h3>Android</h3><p>Screen capture and H.264 encoding foundation; live network streaming is not available yet.</p></div><b>IN DEVELOPMENT</b></article><article><span>03</span><div><h3>iOS</h3><p>ReplayKit capture and companion experience are planned.</p></div><b>PLANNED</b></article></div></section>

    <section className="site-cta" id="get-started"><div className="cta-glow"/><div className="site-section-index">BE PART OF WHAT'S NEXT <span>— 05</span></div><h2>Make every screen<br/><em>feel like your space.</em></h2><p>MirrorX is under active development. Follow the build, explore the foundations and see where the connected workspace goes next.</p><div className="site-hero-actions"><a className="site-button-primary" href="https://github.com/umer714/MirrorX" target="_blank" rel="noreferrer">Explore the project <span>↗</span></a><a className="site-button-text" href="https://github.com/umer714/MirrorX/issues" target="_blank" rel="noreferrer">Follow development <span>↗</span></a></div></section>

    <footer className="site-footer"><a className="site-logo" href="#home"><span className="site-logo-mark">M<span>.</span></span><span>mirror<span>x</span></span></a><span>YOUR PHONE. YOUR PC. ONE EXPERIENCE.</span><div><a href="https://github.com/umer714/MirrorX" target="_blank" rel="noreferrer">GITHUB ↗</a><span>© 2026 MIRRORX</span></div></footer>
  </main>;
}

pub const CSS: &str = r#"
  :root {
    --bg: #0d0d0d;
    --panel: #141414;
    --border: #2a2a2a;
    --accent: #e8ff47;
    --accent2: #ff4757;
    --accent3: #00d2ff;
    --accent4: #b088f9;
    --text: #f0f0f0;
    --muted: #555;
    --transition: 0.6s cubic-bezier(0.77, 0, 0.175, 1);
  }

  body {
    background: var(--bg);
    color: var(--text);
    font-family: "JetBrains Mono", monospace;
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 40px;
    padding: 40px 20px;
    overflow: hidden;
    margin: 0;
  }

  .controls { display: flex; align-items: center; gap: 20px; z-index: 10; }

  .rotate-btn {
    font-family: "Syne", sans-serif;
    font-size: 15px; font-weight: 800;
    letter-spacing: 0.12em; text-transform: uppercase;
    color: var(--bg); background: var(--accent);
    border: none; padding: 14px 36px; cursor: pointer;
    clip-path: polygon(8px 0%, 100% 0%, calc(100% - 8px) 100%, 0% 100%);
    transition: background 0.2s, transform 0.1s;
  }
  .rotate-btn:hover { background: #fff; }
  .rotate-btn:active { transform: scale(0.97); }

  .step-label { font-size: 12px; color: var(--muted); letter-spacing: 0.08em; }
  .step-label span { color: var(--accent); font-weight: 700; }

  .scene { display: flex; align-items: center; justify-content: center; width: 100%; flex: 1; }

  .master-box {
    border: 2px solid var(--border); background: var(--panel);
    padding: 28px; display: grid;
    grid-template-columns: repeat(3, 1fr); gap: 16px;
    width: 560px; transition: transform var(--transition);
    box-shadow: 0 0 80px rgba(0,0,0,0.6); position: relative;
  }
  .master-box::before {
    content: 'MASTER BOX'; position: absolute; top: -11px; left: 20px;
    font-size: 10px; letter-spacing: 0.15em;
    background: var(--accent); color: var(--bg);
    padding: 2px 10px; font-weight: 700;
  }

  .box {
    border: 1px solid var(--border); padding: 18px 14px;
    display: flex; flex-direction: column; gap: 8px;
    position: relative; overflow: hidden; transition: border-color 0.3s;
  }
  .box:hover { border-color: var(--accent); }
  .box::after { content: ""; position: absolute; top: 0; left: 0; width: 3px; height: 100%; }
  .box.c1::after { background: var(--accent); }
  .box.c2::after { background: var(--accent2); }
  .box.c3::after { background: var(--accent3); }
  .box.c4::after { background: var(--accent4); }
  .box.c5::after { background: var(--accent); }
  .box.c6::after { background: var(--accent2); }
  .box.c7::after { background: var(--accent3); }
  .box.c8::after { background: var(--accent4); }
  .box.c9::after { background: var(--accent); }

  .box-label { font-size: 10px; letter-spacing: 0.12em; color: var(--muted); text-transform: uppercase; }
  .box-title { font-family: "Syne", sans-serif; font-size: 15px; font-weight: 700; color: var(--text); line-height: 1.2; }
  .box-value { font-size: 22px; font-weight: 700; color: var(--accent); margin-top: 4px; }
  .box.c2 .box-value { color: var(--accent2); }
  .box.c3 .box-value { color: var(--accent3); }
  .box.c4 .box-value { color: var(--accent4); }
  .box.c5 .box-value { color: var(--accent); }
  .box.c6 .box-value { color: var(--accent2); }
  .box.c7 .box-value { color: var(--accent3); }
  .box.c8 .box-value { color: var(--accent4); }

  .box.wide { grid-column: span 2; }

  .bar-track { width: 100%; height: 3px; background: var(--border); border-radius: 2px; margin-top: 6px; }
  .bar-fill { height: 100%; border-radius: 2px; background: var(--accent); }
  .box.c2 .bar-fill { background: var(--accent2); }
  .box.c3 .bar-fill { background: var(--accent3); }
  .box.c4 .bar-fill { background: var(--accent4); }
"#;

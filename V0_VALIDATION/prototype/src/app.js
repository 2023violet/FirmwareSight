
(function(){
  const F=window.FS_FIXTURES;
  let state=FSState.initial();
  const screen=document.getElementById('screen');
  const modalRoot=document.getElementById('modal-root');
  const live=document.getElementById('live');

  const esc=s=>String(s??'').replace(/[&<>"']/g,c=>({ '&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;' }[c]));
  const fmt=v=>v==null?'—':Number(v).toFixed(1);
  const delta=v=>v===0?'0.0':`${v>0?'+':''}${Number(v).toFixed(1)}`;
  const status=(name,label)=>`<span class="status ${name.toLowerCase()}"><span class="status-icon">${name==='PASS'?'✓':name==='BLOCK'?'×':name==='REVIEW'?'△':name==='UNKNOWN'?'○':'−'}</span>${esc(label||name)}</span>`;
  function announce(s){ live.textContent=s; }
  function setState(action,payload){ state=FSState.transition(state,action,payload); render(); }
  function button(label,cls='',attrs=''){ return `<button class="btn ${cls}" ${attrs}>${label}</button>`; }
  function navActive(){ document.querySelectorAll('[data-nav]').forEach(b=>b.classList.toggle('active',b.dataset.nav===state.route)); }

  function pageHead(title,sub,actions=''){
    return `<div class="page-head"><div><h1 class="page-title">${title}</h1>${sub?`<div class="meta mono">${sub}</div>`:''}</div><div class="actions">${actions}</div></div>`;
  }
  function capability(){
    return `<div class="panel"><div class="cap-grid">
      <div class="cap"><div class="cap-title"><span style="color:var(--pass)">✓</span> ELF ${status('PASS','Loaded & parsed')}</div><div class="cap-detail mono">firmware.elf · 486.2 KB · sha256 a3f9c1e2…</div><button class="linkish" data-route="analyze">Open Analyze →</button></div>
      <div class="cap"><div class="cap-title"><span style="color:${state.mapLoaded?'var(--pass)':'var(--unknown)'}">${state.mapLoaded?'✓':'○'}</span> MAP ${state.mapLoaded?status('PASS','Loaded'):status('UNKNOWN','Not provided')}</div><div class="cap-detail">${state.mapLoaded?'1,284 symbols resolved':'Symbol-level analysis unavailable — sections only'}</div>${state.mapLoaded?'<span class="meta mono">firmware.map</span>':'<button class="linkish" data-add-map>Add .map file… →</button>'}</div>
      <div class="cap"><div class="cap-title"><span style="color:var(--unknown)">○</span> Git ${status('UNKNOWN','Not linked')}</div><div class="cap-detail">No commit provenance for this artifact</div><button class="linkish" disabled>Link repository… →</button></div>
    </div></div>`;
  }

  function overview(){
    const unknown=FSState.unknownCount(state), pending=FSState.pendingReviews(state), blocks=FSState.effectiveBlocks(state);
    const verdict=FSState.overall(state);
    const summary=verdict==='BLOCK'?'Not yet — 1 rule failed, 1 review pending.':verdict==='REVIEW'?'Needs review before release.':'Configured release policy is satisfied.';
    return `<div class="page">${pageHead('Overview','relay-controller · artifact v0.3.0-rc2 · simulated V0 fixture',
      `${button('⇄ Compare with…','', 'data-route="compare"')} ${button('◇ Re-run Release Gate','primary','data-route="gate"')}`)}
      ${capability()}
      <div class="panel" style="margin-top:14px">
        <div class="panel-head"><div class="panel-title">Can we ship now?</div><div class="meta">Configured FirmwareSight release policy only</div></div>
        <div class="readiness"><div class="readiness-title">${status(verdict)} <span>${summary}</span></div>
          <div class="fact-list">
            ${blocks?`<div class="fact-row"><span class="fact-icon block">×</span><div><code>signed_image.required</code> failed — image signature header not found.</div></div>`:''}
            ${pending?`<div class="fact-row"><span class="fact-icon review">△</span><div><code>ota_staging.new_section</code> pending review — new +24.0 KB flash section.</div></div>`:''}
            ${unknown?`<div class="fact-row"><span class="fact-icon unknown">○</span><div>${unknown} rules cannot be evaluated without a MAP file: <code>map_parity</code>, <code>symbol_audit</code>.</div></div>`:''}
            ${!blocks&&!pending&&!unknown?`<div class="fact-row"><span style="color:var(--pass)">✓</span><div>All effective blockers are resolved. Accepted reviews remain auditable as REVIEW findings.</div></div>`:''}
          </div>
          <div class="next-row"><strong>Next steps</strong>
            ${blocks?button('Re-run gate','primary','data-route="gate"'):''}
            ${pending?button('Review OTA section','','data-route="gate"'):''}
            ${!state.mapLoaded?button('Add .map file…','ghost','data-add-map'):''}
            <span class="push meta">${blocks?'Blocked builds never produce a bundle':'Readiness uses configured policy, not legal/safety certification'}</span>
          </div>
        </div>
      </div>
      <div class="panel" style="margin-top:14px"><div class="kpis">
        <div class="kpi"><div class="kpi-label">FLASH USED</div><div class="kpi-value mono">486.2 KB</div><div class="meta">of 512.0 KB budget · 95.0% used</div></div>
        <div class="kpi"><div class="kpi-label">LARGEST SECTION</div><div class="kpi-value mono">.text · 262.4 KB</div><div class="meta">54.0% of flash</div></div>
        <div class="kpi"><div class="kpi-label">SYMBOLS</div><div class="kpi-value mono">${state.mapLoaded?'1,284':'—'}</div><div class="meta">${state.mapLoaded?'resolved from MAP':'requires MAP file'}</div></div>
        <div class="kpi"><div class="kpi-label">LAST GATE</div><div class="kpi-value mono">${verdict}</div><div class="meta">prototype state</div></div>
      </div></div>
    </div>`;
  }

  function analyze(){
    const tabs=['sections','symbols','dependencies'].map(t=>`<button class="tab ${state.analyzeTab===t?'active':''}" data-tab="${t}">${t[0].toUpperCase()+t.slice(1)}</button>`).join('');
    const actions=`<button class="btn" disabled title="Export is not available in this V0 prototype">Export CSV</button>${button('⇄ Open Compare','primary','data-route="compare"')}`;
    let body='';
    if(state.analyzeTab==='sections') body=sectionsView();
    if(state.analyzeTab==='symbols') body=symbolsView();
    if(state.analyzeTab==='dependencies') body=dependenciesView();
    return `<div class="page">${pageHead('Analyze','',actions)}
      <div class="tabs">${tabs}</div>
      <div class="chips"><span class="chip">✓ ELF <code>firmware.elf</code></span><span class="chip">${state.mapLoaded?'✓':'○'} MAP ${state.mapLoaded?'<code>firmware.map</code>':'not provided'}</span><span class="chip">○ Git not linked</span></div>
      <div style="height:12px"></div>${body}
    </div>`;
  }

  function sectionsView(){
    const secs=F.builds.sections.filter(x=>x.new!=null && x.name!=='.legacy_log');
    const rows=secs.map((s,i)=>{
      const share=(s.new/486.2*100);
      return `<tr class="clickable ${state.selectedSection===s.name?'selected':''}" data-section="${esc(s.name)}"><td><code>${esc(s.name)}</code></td><td class="num">${fmt(s.new)}</td><td class="num">${share.toFixed(1)}%</td><td class="num delta ${s.delta>0?'pos':s.delta<0?'neg':''}">${delta(s.delta)}</td><td><div class="bar"><span style="width:${Math.min(100,share/54*100).toFixed(0)}%"></span></div></td></tr>`;
    }).join('');
    return `<div class="two-col"><div class="main-col"><div class="panel"><div class="panel-head"><div class="panel-title">Sections <span class="status unknown">ELF · 8 flash sections</span></div><div class="meta">row selected · open full table</div></div>
      <table><thead><tr><th>Section</th><th class="num">Size (KB)</th><th class="num">Share</th><th class="num">Δ vs v0.2.1</th><th>Size distribution</th></tr></thead><tbody>${rows}</tbody></table>
      <div class="meta" style="padding:8px 16px">Debug sections excluded from flash budget · accounting is demo data.</div></div>
      ${state.mapLoaded?symbolsTableMini():`<div class="panel"><div class="panel-head"><div class="panel-title">Symbols</div><div class="meta">MAP required</div></div><div class="empty-symbols"><strong>Symbol-level analysis is unavailable.</strong>Add <code>firmware.map</code> to resolve symbol evidence.<br><br>${button('Add .map file…','primary','data-add-map')}</div></div>`}
      </div>${sectionInspector()}</div>`;
  }
  function symbolsTableMini(){
    const rows=F.builds.symbols.map(s=>`<tr><td><code>${s.symbol}</code></td><td><code>${s.section}</code></td><td class="num">${fmt(s.size)}</td><td class="num delta ${s.delta>0?'pos':s.delta<0?'neg':''}">${delta(s.delta)}</td><td><div class="bar"><span style="width:${Math.min(100,s.size/18.2*100).toFixed(0)}%"></span></div></td></tr>`).join('');
    return `<div class="panel"><div class="panel-head"><div class="panel-title">Symbols <span class="status pass">MAP · 1,284 resolved</span></div><div class="meta">top 6 by size</div></div><table><thead><tr><th>Symbol</th><th>Section</th><th class="num">Size (KB)</th><th class="num">Δ (KB)</th><th>Size</th></tr></thead><tbody>${rows}</tbody></table></div>`;
  }
  function sectionInspector(){
    const s=F.builds.sections.find(x=>x.name===state.selectedSection)||F.builds.sections[1];
    const share=s.new? (s.new/486.2*100).toFixed(1):'—';
    return `<aside class="panel inspector"><div class="panel-head"><div class="panel-title"><code>${esc(s.name)}</code></div>${status('PASS','Observed')}</div><div class="inspector-body">
      <div class="kv"><span>Size</span><code>${s.new==null?'—':fmt(s.new)+' KB'}</code></div>
      <div class="kv"><span>Share of flash</span><code>${share}%</code></div>
      <div class="kv"><span>Δ vs v0.2.1</span><code class="${s.delta>0?'delta pos':s.delta<0?'delta neg':''}">${delta(s.delta)} KB</code></div>
      <div class="kv"><span>Source</span><code>${state.mapLoaded?'ELF headers + firmware.map':'ELF section headers'}</code></div>
      <div class="section-label">TOP CONTRIBUTORS</div>
      ${state.mapLoaded?`<div class="kv"><code>mqtt_task</code><code>18.2</code></div><div class="kv"><code>tls_handshake</code><code>12.9</code></div><div class="kv"><code>ota_update_apply</code><code>11.4</code></div>`:'<div class="meta">Add MAP to resolve symbol contributors.</div>'}
      <div class="callout">Observed/Derived demo evidence only. Nothing is inferred beyond the fixture narrative.</div>
    </div></aside>`;
  }

  function symbolsView(){
    if(!state.mapLoaded) return `<div class="panel"><div class="panel-head"><div class="panel-title">Symbols</div>${status('UNKNOWN','MAP required')}</div><div class="empty-symbols"><strong>No symbol evidence yet.</strong>Sections remain available from ELF, but symbol-level analysis requires a MAP file.<br><br>${button('Add .map file…','primary','data-add-map')}</div></div>`;
    return `<div class="panel"><div class="panel-head"><div class="panel-title">Symbols <span class="status pass">1,284 resolved</span></div><div class="meta mono">source: firmware.map</div></div>${symbolsTableMini().replace(/^<div class="panel">|<\/div>$/g,'')}</div>`;
  }

  function dependenciesView(){
    if(!state.mapLoaded) return `<div class="panel"><div class="panel-head"><div class="panel-title">Dependencies</div>${status('UNKNOWN','MAP required')}</div><div class="empty-symbols"><strong>Dependency fingerprints are unavailable.</strong>Add MAP evidence before using this view.<br><br>${button('Add .map file…','primary','data-add-map')}</div></div>`;
    const rows=F.dependencies.map(d=>{
      const declared=state.dependencyDeclarations[d.component];
      const version=declared||d.version;
      const stat=declared&&d.status==='Unknown'?'Declared':d.status;
      const cls=stat==='Observed'?'PASS':stat==='Declared'?'PASS':'UNKNOWN';
      return `<tr class="clickable ${state.selectedDependency===d.component?'selected':''}" data-dep="${esc(d.component)}"><td><strong>${esc(d.component)}</strong></td><td><code>${version||'—'}</code></td><td>${status(cls,stat)}</td><td class="muted">${esc(d.evidence)}</td><td>${stat==='Unknown'?button('Declare version…','','data-declare="'+esc(d.component)+'"'):'—'}</td></tr>`;
    }).join('');
    return `<div class="two-col"><div class="main-col"><div class="panel"><div class="panel-head"><div class="panel-title">Dependencies <span class="status unknown">5 detected or declared</span></div><div class="meta">detection: strings + symbol fingerprints · <code>firmware.map</code></div></div>
      <table><thead><tr><th>Component</th><th>Version</th><th>Status</th><th>Evidence</th><th>Next</th></tr></thead><tbody>${rows}</tbody></table>
      <div class="meta" style="padding:8px 16px">Observed = detected by FirmwareSight · Declared = asserted by you, recorded with author + time · Unknown remains explicit.</div></div></div>${dependencyInspector()}</div>`;
  }
  function dependencyInspector(){
    const name=state.selectedDependency||'mbedTLS';
    const d=F.dependencies.find(x=>x.component===name)||F.dependencies[2];
    const declared=state.dependencyDeclarations[name];
    const stat=declared&&d.status==='Unknown'?'Declared':d.status;
    return `<aside class="panel inspector"><div class="panel-head"><div class="panel-title">${esc(name)}</div>${status(stat==='Observed'||stat==='Declared'?'PASS':'UNKNOWN',stat)}</div><div class="inspector-body">
      <div class="section-label">WHAT WE DETECTED</div>
      ${name==='mbedTLS'?`<div class="kv"><span>TLS symbols in .text</span><code>14</code></div><div class="kv"><span>Version strings found</span><code>0</code></div><div class="kv"><span>Match type</span><code>fingerprint</code></div>`:`<div class="kv"><span>Evidence</span><span>${esc(d.evidence)}</span></div>`}
      <div class="section-label">WHAT WE DO NOT KNOW</div>
      ${stat==='Unknown'?`<div class="kv"><span>Version</span><code>unknown</code></div><div class="kv"><span>License</span><code>unknown</code></div><div class="kv"><span>Build provenance</span><code>unknown</code></div>`:`<div class="meta">Manual declaration supplies a version assertion; it does not convert detection to Observed.</div>`}
      <div class="callout">We report what we detect — we never guess a version. Declared evidence is stored separately from Observed facts.</div>
      <div style="margin-top:12px">${stat==='Unknown'?button('Declare version…','','data-declare="'+esc(name)+'"'):''}</div>
    </div></aside>`;
  }

  function compare(){
    const rows=F.builds.sections.map(s=>`<tr class="${s.change==='Added'?'diff-added':s.change==='Removed'?'diff-removed':s.change==='Changed'?'diff-changed':''}"><td><code>${esc(s.name)}</code></td><td class="num">${fmt(s.old)}</td><td class="num">${fmt(s.new)}</td><td class="num delta ${s.delta>0?'pos':s.delta<0?'neg':''}">${delta(s.delta)}</td><td>${s.change==='Added'?status('PASS','Added'):s.change==='Removed'?status('BLOCK','Removed'):s.change==='Changed'?'<span class="status" style="color:var(--accent)">○ Changed</span>':'<span class="status">Unchanged</span>'}</td></tr>`).join('');
    return `<div class="page">${pageHead('Compare','v0.2.1 → v0.3.0-rc2',`<button class="btn" disabled title="Export is not available in this V0 prototype">Export diff (.json)</button>`)}
      <div class="summary-strip"><div>Flash: <code>474.8 KB → <strong>486.2 KB</strong> <span class="delta pos">+11.4 KB · +2.4%</span></code> · budget <code>512.0 KB</code></div><div>${status('PASS','size_budget.hard · PASS')} ${status('REVIEW','flash_budget.soft · REVIEW')}</div></div>
      <div class="two-col"><div class="panel"><table><thead><tr><th>Section</th><th class="num">v0.2.1 (KB)</th><th class="num">v0.3.0-rc2 (KB)</th><th class="num">Δ (KB)</th><th>Change</th></tr></thead><tbody>${rows}</tbody></table><div class="meta" style="padding:8px 16px">— = section absent in that artifact; never shown as 0.</div></div>
      <aside class="panel inspector"><div class="panel-head"><div class="panel-title">Biggest growth contributors</div></div><div class="inspector-body">
        <div class="kv"><code>.ota_staging (new)</code><code class="delta pos">+24.0</code></div>
        <div class="kv"><code>mqtt_task (.text)</code><code class="delta pos">+2.4</code></div>
        <div class="kv"><code>.rodata</code><code class="delta pos">+1.2</code></div>
        <div class="kv"><code>.data</code><code class="delta neg">-9.5</code></div>
        <div class="kv"><code>.legacy_log (removed)</code><code class="delta neg">-6.2</code></div>
        <div class="callout">${state.mapLoaded?'Symbol deltas use current MAP evidence.':'Symbol detail is unavailable until MAP is provided.'} Net: +11.4 KB.</div>
        <div style="margin-top:12px">${button('Open Release Gate','primary','data-route="gate"')}</div>
      </div></aside></div>
    </div>`;
  }

  function gate(){
    const overall=FSState.overall(state), can=FSState.canBuildBundle(state);
    const block=FSState.effectiveBlocks(state);
    const pending=FSState.pendingReviews(state);
    return `<div class="page">${pageHead('Release Gate',`run #${12+state.gateRerunCount} · target v0.3.0-rc2`,button('◇ Re-run gate','primary','data-rerun'))}
      <div class="summary-strip"><div style="display:flex;gap:10px;align-items:center">${status(overall)}<strong>${overall==='BLOCK'?'Blocked':overall==='REVIEW'?'Review required':'Ready under configured policy'} — ${block} block, ${pending} review pending, ${FSState.unknownCount(state)} unknown.</strong></div><div class="meta">finding state is preserved separately from effective severity</div></div>
      <div class="gate-layout"><div class="panel">
        ${block?`<div class="group-head"><span>× BLOCK · 1</span><span>must be fixed before a ready bundle</span></div><div class="finding"><div class="finding-title"><code>signed_image.required</code> ${status('BLOCK','FAIL')}</div><div class="finding-detail"><strong>Why</strong><span>Signature header not found — expected SIGv2 magic.</span><strong>Evidence</strong><span><code>image sha256 a3f9c1e2… · bytes 0x0200–0x0207 = FF…</code></span><strong>Next</strong><span>Run signing step, then re-run Gate.</span></div><div class="finding-actions">${button('Apply simulated signing fix','','data-fix-signature')}</div></div>`:''}
        <div class="group-head"><span>△ REVIEW · 2</span><span>explicit acceptance only — never silent</span></div>
        <div class="finding"><div class="finding-title"><code>flash_budget.soft</code> ${status('PASS','Accepted')}</div><div class="meta">Flash 486.2 KB / 512.0 KB (95.0%) · historical acceptance retained.</div></div>
        <div class="finding"><div class="finding-title"><code>ota_staging.new_section</code> ${state.otaAccepted?status('PASS','Accepted'):status('REVIEW','REVIEW')}</div><div class="meta">New flash section (+24.0 KB) — confirm it belongs in the release map.</div>${state.otaAccepted?`<div class="meta mono">finding_id f-ota-section · original_state REVIEW · acceptance recorded</div>`:`<div class="finding-actions">${button('Accept review…','','data-accept-review')}</div>`}</div>
        <div class="group-head"><span>○ UNKNOWN · ${FSState.unknownCount(state)}</span><span>evidence missing — state is not rewritten</span></div>
        ${state.mapLoaded?`<div class="finding"><div class="meta">No current MAP-dependent unknowns. <code>map_parity</code> and <code>symbol_audit</code> re-evaluated to PASS.</div></div>`:`<div class="finding"><div class="finding-title"><code>map_parity</code> ${status('UNKNOWN')}</div><div class="meta">MAP not provided — cannot evaluate.</div><div class="finding-actions">${button('Add .map file…','ghost','data-add-map')}</div></div><div class="finding"><div class="finding-title"><code>symbol_audit</code> ${status('UNKNOWN')}</div><div class="meta">Symbol evidence unavailable without MAP.</div></div>`}
        <div class="group-head"><span>✓ PASS · ${state.mapLoaded?6:4}</span><span>deterministically evaluated</span></div>
        <div class="finding"><div class="meta"><code>entry_point.valid</code> · <code>size_budget.hard</code> · <code>elf.structure</code> · <code>version.match</code>${state.mapLoaded?' · <code>map_parity</code> · <code>symbol_audit</code>':''}</div></div>
        <div class="group-head"><span>− N/A · 1</span><span>rule not applicable</span></div>
        <div class="finding"><div class="meta"><code>sbom.present</code> — N/A; no SBOM input configured for this prototype artifact.</div></div>
      </div>
      <aside class="panel"><div class="panel-head"><div class="panel-title">Bundle preview</div><div class="meta">built only when no effective blocker remains</div></div><div class="bundle-list">
        ${['firmware.elf','manifest.json','gate_results.json','accepted_reviews.json','environment.txt'].map(f=>`<div class="bundle-file"><span>▱</span><code>${f}</code><span class="right">${f==='accepted_reviews.json'?(state.acceptanceRecords.length+1)+' accepted':'✓'}</span></div>`).join('')}
      </div><div style="padding:12px 16px;border-top:1px solid var(--border)">${button('Build bundle','primary',`data-build-bundle ${can?'':'disabled'}`)}<div class="meta" style="margin-top:8px">${can?'Configured Gate allows bundle creation.':'Resolve BLOCK / pending REVIEW / required evidence first.'}</div></div></aside>
      </div>
    </div>`;
  }

  function history(){
    const currentBundle=state.bundleBuilt?`<tr class="row-selected"><td><code>bnd_v0_current</code></td><td>Bundle</td><td class="mono">2026-09-27 12:05</td><td>V0 prototype</td><td>${status('PASS')}</td><td class="mono">a3f9c1e2</td><td class="num">+2.4%</td></tr>`:'';
    return `<div class="page">${pageHead('Bundle & History','local prototype history · evidence package view',`<button class="btn" disabled title="Export is not available in this V0 prototype">Export…</button>`)}
      <div class="history-layout"><div class="panel"><div class="panel-head"><div class="panel-title">History</div><div class="meta">hashes truncated in table</div></div><table><thead><tr><th>ID</th><th>Type</th><th>Created (UTC)</th><th>Trigger</th><th>Verdict</th><th>Artifact</th><th class="num">Δ flash</th></tr></thead><tbody>
        ${currentBundle}
        <tr><td><code>run #12</code></td><td>Gate run</td><td class="mono">2026-09-26 18:47</td><td>UI · after Analyze</td><td>${status('BLOCK')}</td><td class="mono">a3f9c1e2</td><td class="num">+2.4%</td></tr>
        <tr><td><code>bnd_01j9q4x2</code></td><td>Bundle</td><td class="mono">2026-08-14 09:31</td><td>release v0.2.1</td><td>${status('PASS')}</td><td class="mono">77c1d9b0</td><td class="num">−0.6%</td></tr>
        <tr><td><code>run #09</code></td><td>Gate run</td><td class="mono">2026-08-14 09:28</td><td>release v0.2.1</td><td>${status('PASS')}</td><td class="mono">77c1d9b0</td><td class="num">−0.6%</td></tr>
      </tbody></table></div>
      <aside class="panel"><div class="panel-head"><div class="panel-title"><code>${state.bundleBuilt?'bnd_v0_current':'bnd_01j9q4x2'}</code></div>${status('PASS')}</div><div class="inspector-body">
        <div class="kv"><span>Artifact</span><code>${state.bundleBuilt?'v0.3.0-rc2 · a3f9c1e2':'v0.2.1 · 77c1d9b0'}</code></div>
        <div class="section-label">CONTENTS</div>
        ${['firmware.elf','manifest.json','gate_results.json','accepted_reviews.json','environment.txt'].map(f=>`<div class="bundle-file"><span>▱</span><code>${f}</code><span class="right">${f==='firmware.elf'?'sha256 + size':'evidence file'}</span></div>`).join('')}
        <div class="callout">This is a Release Evidence Package: hashes and machine-readable records allow independent verification without relying on a screenshot.</div>
      </div></aside></div>
    </div>`;
  }

  function parseFailure(){
    return `<div class="page">${pageHead('Analyze','Candidate replacement failed',button('Open last good artifact','','data-recover'))}
      <div class="error-card"><div class="error-title">${status('BLOCK','Parse failed')}<span>Could not parse <code>invalid-firmware.elf</code> as an ELF image.</span></div>
      <div class="chips"><span class="chip"><code>invalid-firmware.elf · 412 KB</code></span><span class="chip">candidate only</span></div>
      <div class="error-grid">
        <div class="error-row"><div class="error-label">What happened</div><div>The parser stopped at the ELF header sanity check — this candidate is not an ELF binary.</div></div>
        <div class="error-row"><div class="error-label">Why we know</div><div>Header magic at <code>offset 0x00</code> is <code>64 65 6D 6F</code> (“demo”) — expected <code>7F 45 4C 46</code>.</div></div>
        <div class="error-row"><div class="error-label">What to do</div><div>Choose the linker output, not a renamed dump. ${button('Choose last good artifact','primary','data-recover')}</div></div>
        <div class="error-row"><div class="error-label">Diagnostics</div><div><code>ERR-PARSE-2041 · elf/parse_header:88</code> ${button('Copy diagnostics','','data-copy-diagnostics')}</div></div>
      </div>
      <div class="callout">Nothing was overwritten — workspace still holds Last Good Artifact <code>v0.3.0-rc2 · a3f9c1e2…</code> and the previously loaded MAP evidence.</div>
      </div>
    </div>`;
  }

  function activity(){
    return `<div class="page">${pageHead('Recent activity','prototype-only event history')}<div class="activity-list">${state.activity.map(x=>`<div class="activity-row">${esc(x)}</div>`).join('')}</div></div>`;
  }

  function render(){
    navActive();
    let html='';
    if(state.route==='overview') html=overview();
    else if(state.route==='analyze') html=analyze();
    else if(state.route==='compare') html=compare();
    else if(state.route==='gate') html=gate();
    else if(state.route==='history') html=history();
    else if(state.route==='parse-failure') html=parseFailure();
    else if(state.route==='activity') html=activity();
    screen.innerHTML=html;
    bind();
  }

  function bind(){
    screen.querySelectorAll('[data-route]').forEach(el=>el.addEventListener('click',()=>setState('NAVIGATE',{route:el.dataset.route})));
    document.querySelectorAll('[data-nav]').forEach(el=>el.onclick=()=>setState('NAVIGATE',{route:el.dataset.nav}));
    screen.querySelectorAll('[data-tab]').forEach(el=>el.addEventListener('click',()=>setState('ANALYZE_TAB',{tab:el.dataset.tab})));
    screen.querySelectorAll('[data-add-map]').forEach(el=>el.addEventListener('click',()=>{ setState('ADD_MAP'); announce('firmware.map added. Symbol analysis is now available.'); }));
    screen.querySelectorAll('[data-section]').forEach(el=>el.addEventListener('click',()=>setState('SELECT_SECTION',{name:el.dataset.section})));
    screen.querySelectorAll('[data-dep]').forEach(el=>el.addEventListener('click',()=>setState('SELECT_DEP',{name:el.dataset.dep})));
    screen.querySelectorAll('[data-declare]').forEach(el=>el.addEventListener('click',e=>openDeclare(e.currentTarget.dataset.declare)));
    screen.querySelectorAll('[data-accept-review]').forEach(el=>el.addEventListener('click',openAcceptReview));
    screen.querySelectorAll('[data-fix-signature]').forEach(el=>el.addEventListener('click',()=>{setState('FIX_SIGNATURE');announce('Prototype-only signing fix applied.');}));
    screen.querySelectorAll('[data-rerun]').forEach(el=>el.addEventListener('click',()=>{setState('RERUN_GATE');announce('Release Gate re-run.');}));
    screen.querySelectorAll('[data-build-bundle]').forEach(el=>el.addEventListener('click',()=>{if(FSState.canBuildBundle(state)){setState('BUILD_BUNDLE');announce('Release Evidence Package created.');}}));
    screen.querySelectorAll('[data-invalid-elf]').forEach(el=>el.addEventListener('click',()=>setState('INVALID_ELF')));
    screen.querySelectorAll('[data-recover]').forEach(el=>el.addEventListener('click',()=>setState('RECOVER_ELF')));
    screen.querySelectorAll('[data-copy-diagnostics]').forEach(el=>el.addEventListener('click',()=>{navigator.clipboard?.writeText('ERR-PARSE-2041 · elf/parse_header:88');announce('Diagnostics copied.');}));
    // Prototype utility: invalid candidate action appears on Analyze via keyboard shortcut and page utility.
    if(state.route==='analyze'){
      const actions=screen.querySelector('.actions');
      if(actions && !actions.querySelector('[data-invalid-elf]')){
        const b=document.createElement('button'); b.className='btn'; b.dataset.invalidElf=''; b.textContent='Replace with invalid ELF…';
        b.addEventListener('click',()=>setState('INVALID_ELF')); actions.prepend(b);
      }
    }
  }

  let lastFocus=null;
  function trapModal(e){
    if(e.key==='Escape'){ closeModal(); return; }
    if(e.key!=='Tab') return;
    const focusables=[...modalRoot.querySelectorAll('button,input,[href],[tabindex]:not([tabindex="-1"])')].filter(x=>!x.disabled);
    if(!focusables.length) return;
    const first=focusables[0], last=focusables[focusables.length-1];
    if(e.shiftKey && document.activeElement===first){e.preventDefault();last.focus();}
    else if(!e.shiftKey && document.activeElement===last){e.preventDefault();first.focus();}
  }
  function closeModal(){ modalRoot.innerHTML=''; document.removeEventListener('keydown',trapModal); lastFocus?.focus(); }
  function modal(title,body,actions){
    lastFocus=document.activeElement;
    modalRoot.innerHTML=`<div class="modal-backdrop"><div class="modal" role="dialog" aria-modal="true" aria-labelledby="modal-title"><div class="modal-head" id="modal-title">${title}</div><div class="modal-body">${body}</div><div class="modal-actions">${actions}</div></div></div>`;
    document.addEventListener('keydown',trapModal);
    modalRoot.querySelector('input,button')?.focus();
    modalRoot.querySelector('.modal-backdrop').addEventListener('mousedown',e=>{if(e.target===e.currentTarget) closeModal();});
  }
  function openDeclare(name){
    modal('Declare component version',`<label>Version or build tag<input id="dep-version" class="field" placeholder="e.g. 3.5.2"></label><div class="help">Declared entries are labeled “Declared” — never mixed with Observed facts.</div>`,
      `<button class="btn" data-close>Cancel</button><button class="btn primary" data-confirm>Declare</button>`);
    modalRoot.querySelector('[data-close]').onclick=closeModal;
    modalRoot.querySelector('[data-confirm]').onclick=()=>{
      const v=modalRoot.querySelector('#dep-version').value.trim();
      if(!v){modalRoot.querySelector('#dep-version').focus();return;}
      closeModal(); setState('DECLARE_DEP',{name,version:v}); announce(name+' '+v+' recorded as Declared evidence.');
    };
  }
  function openAcceptReview(){
    modal('Accept review',`<div><code>finding_id: f-ota-section</code></div><div><code>original_state: REVIEW</code></div><label style="display:block;margin-top:12px">Reason<input id="review-reason" class="field" value="Confirmed expected OTA staging section."></label><div class="help">Acceptance creates an immutable disposition record. The original finding remains REVIEW.</div>`,
      `<button class="btn" data-close>Cancel</button><button class="btn primary" data-confirm>Accept review</button>`);
    modalRoot.querySelector('[data-close]').onclick=closeModal;
    modalRoot.querySelector('[data-confirm]').onclick=()=>{
      const reason=modalRoot.querySelector('#review-reason').value.trim();
      if(!reason){modalRoot.querySelector('#review-reason').focus();return;}
      closeModal(); setState('ACCEPT_OTA',{actor:'V0 participant',reason}); announce('Review acceptance recorded; finding remains REVIEW.');
    };
  }

  window.addEventListener('keydown',e=>{
    if(e.altKey&&e.shiftKey&&e.key.toLowerCase()==='r'){ state=FSState.initial(); render(); announce('Prototype reset to STATE A.');}
  });
  render();
})();

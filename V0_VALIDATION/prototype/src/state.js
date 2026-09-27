(function(root,factory){
  const api=factory();
  if(typeof module==='object'&&module.exports) module.exports=api;
  root.FSState=api;
})(typeof window!=='undefined'?window:globalThis,function(){
  function initial(){
    return {
      route:'overview', analyzeTab:'sections',
      mapLoaded:false, parseFailure:false, candidateFile:null,
      selectedSection:'.text', selectedDependency:'mbedTLS',
      dependencyDeclarations:{FreeRTOS:'10.4.0'},
      otaAccepted:false, signatureFixed:false, gateRerunCount:0,
      bundleBuilt:false, acceptanceRecords:[],
      activity:['Prototype opened in STATE A — MAP not provided.']
    };
  }
  function clone(s){ return JSON.parse(JSON.stringify(s)); }
  function transition(s, action, payload={}){
    const n=clone(s);
    switch(action){
      case 'NAVIGATE': n.route=payload.route; break;
      case 'ANALYZE_TAB': n.route='analyze'; n.analyzeTab=payload.tab; break;
      case 'ADD_MAP':
        n.mapLoaded=true; n.route='analyze'; n.analyzeTab='symbols';
        n.activity.unshift('firmware.map added — 1,284 symbols resolved; MAP-dependent rules re-evaluated.');
        break;
      case 'SELECT_SECTION': n.selectedSection=payload.name; break;
      case 'SELECT_DEP': n.selectedDependency=payload.name; break;
      case 'DECLARE_DEP':
        n.dependencyDeclarations[payload.name]=payload.version;
        n.activity.unshift(payload.name+' '+payload.version+' recorded as Declared evidence.');
        break;
      case 'ACCEPT_OTA':
        if(!n.otaAccepted){
          n.otaAccepted=true;
          n.acceptanceRecords.push({
            finding_id:'f-ota-section', original_state:'REVIEW',
            actor:payload.actor||'V0 participant', timestamp:'2026-09-27T12:00:00Z',
            reason:payload.reason||'Confirmed expected OTA staging section.'
          });
          n.activity.unshift('Review acceptance recorded for ota_staging.new_section; finding remains REVIEW.');
        }
        break;
      case 'FIX_SIGNATURE':
        n.signatureFixed=true;
        n.activity.unshift('Prototype-only signing fix applied. Re-run Gate to update readiness.');
        break;
      case 'RERUN_GATE':
        n.gateRerunCount+=1;
        n.activity.unshift('Release Gate re-run with current evidence.');
        break;
      case 'BUILD_BUNDLE':
        if(canBuildBundle(n)){
          n.bundleBuilt=true; n.route='history';
          n.activity.unshift('Prototype Release Evidence Package created.');
        }
        break;
      case 'INVALID_ELF':
        n.parseFailure=true; n.candidateFile='invalid-firmware.elf'; n.route='parse-failure';
        n.activity.unshift('Candidate invalid-firmware.elf failed parse; Last Good Artifact preserved.');
        break;
      case 'RECOVER_ELF':
        n.parseFailure=false; n.candidateFile=null; n.route='analyze';
        n.activity.unshift('Recovered to Last Good Artifact firmware.elf.');
        break;
      case 'RESET': return initial();
    }
    return n;
  }
  function pendingReviews(s){ return s.otaAccepted?0:1; }
  function effectiveBlocks(s){ return s.signatureFixed?0:1; }
  function unknownCount(s){ return s.mapLoaded?0:2; }
  function canBuildBundle(s){ return effectiveBlocks(s)===0 && pendingReviews(s)===0 && unknownCount(s)===0; }
  function overall(s){
    if(effectiveBlocks(s)>0) return 'BLOCK';
    if(pendingReviews(s)>0) return 'REVIEW';
    if(unknownCount(s)>0) return 'REVIEW';
    return 'PASS';
  }
  return {initial,transition,pendingReviews,effectiveBlocks,unknownCount,canBuildBundle,overall};
});

const assert=require('assert');
const S=require('../prototype/src/state.js');
let s=S.initial();
assert.equal(s.mapLoaded,false);
assert.equal(S.unknownCount(s),2);
assert.equal(S.overall(s),'BLOCK');
assert.equal(S.canBuildBundle(s),false);

s=S.transition(s,'ADD_MAP');
assert.equal(s.mapLoaded,true);
assert.equal(S.unknownCount(s),0);

s=S.transition(s,'ACCEPT_OTA',{actor:'dry-run',reason:'expected OTA staging'});
assert.equal(s.otaAccepted,true);
assert.equal(s.acceptanceRecords.length,1);
assert.equal(s.acceptanceRecords[0].original_state,'REVIEW');

s=S.transition(s,'FIX_SIGNATURE');
s=S.transition(s,'RERUN_GATE');
assert.equal(S.effectiveBlocks(s),0);
assert.equal(S.pendingReviews(s),0);
assert.equal(S.overall(s),'PASS');
assert.equal(S.canBuildBundle(s),true);

s=S.transition(s,'BUILD_BUNDLE');
assert.equal(s.bundleBuilt,true);
assert.equal(s.route,'history');

s=S.transition(s,'INVALID_ELF');
assert.equal(s.parseFailure,true);
assert.equal(s.route,'parse-failure');
assert.equal(s.mapLoaded,true);

s=S.transition(s,'RECOVER_ELF');
assert.equal(s.parseFailure,false);
assert.equal(s.mapLoaded,true);
assert.equal(s.route,'analyze');

console.log('state-machine dry run: PASS');

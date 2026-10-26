"""Measure same-machine clean guest rebuild identity, never cross-machine assume."""
from pathlib import Path
import hashlib,json,os,subprocess,tempfile
ROOT=Path(__file__).resolve().parents[2]
report=ROOT/'target/abi-evidence/vslice';report.mkdir(parents=True,exist_ok=True)
original=ROOT/'target/abi-guests/wasm32-unknown-unknown/release/mortgage_headless.wasm'
first=hashlib.sha256(original.read_bytes()).hexdigest()
with tempfile.TemporaryDirectory(prefix='mortgage-clean-',dir=ROOT/'target') as target:
    env=os.environ.copy();env['CARGO_TARGET_DIR']=target;env['CARGO_COMPONENT_CACHE_DIR']=str(ROOT/'target/abi-component-cache')
    command=['cargo','component','build','--manifest-path','examples/wasm-components/Cargo.toml','-p','mortgage-headless','--target','wasm32-unknown-unknown','--release','--locked']
    with (report/'rebuild.txt').open('w') as output:
        subprocess.run(command,cwd=ROOT,env=env,stdout=output,stderr=subprocess.STDOUT,check=True)
    artifact=Path(target)/'wasm32-unknown-unknown/release/mortgage_headless.wasm'
    clean=hashlib.sha256(artifact.read_bytes()).hexdigest()
    result={'target':'wasm32-unknown-unknown','firstSha256':first,'cleanSha256':clean,'sameMachine':True,'cleanTargetDirectory':True,'command':command}
    (ROOT/'target/mortgage-rebuild.json').write_text(json.dumps(result,indent=2)+'\n')
    assert first==clean,'Same-machine rebuild differs; inspect evidence instead of accepting assumed reproducibility'
print('PASS: same-machine clean mortgage rebuild SHA-256:',first)

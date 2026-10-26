"""Stamp complete local packages from actual locked Component Model builds."""
from pathlib import Path
import hashlib,json,shutil
ROOT=Path(__file__).resolve().parents[2]
WB='https://workbench.example/ns/0.1#'

def package(name,artifact,observer=False):
    out=ROOT/'target/mortgage-packages'/name
    if out.exists():shutil.rmtree(out)
    (out/'schemas').mkdir(parents=True)
    source=ROOT/'target/abi-guests/wasm32-unknown-unknown/release'/artifact
    shutil.copyfile(source,out/'logic.wasm')
    digest=hashlib.sha256(source.read_bytes()).hexdigest()
    for direction in ['input','output']:
        source_direction='output' if observer else direction
        shutil.copyfile(ROOT/f'examples/mortgage-headless/schemas/mortgage-{source_direction}.schema.json',out/f'schemas/mortgage-{direction}.schema.json')
    identifier=f'org.workbench.example.{name}'
    m={'manifestVersion':'0.1','id':identifier,'version':'0.1.0','runtime':{'kind':'wasm-component','interface':'org.workbench.component@0.1','module':'logic.wasm','sha256':digest},'metadata':'capabilities.ttl','permissions':[],
       'ports':{d:[{'name':port,'schema':f'schemas/mortgage-{s}.schema.json','semanticType':WB+sem,'unit':WB+'GBP'}] for d,port,s,sem in [('inputs','payment' if observer else 'calculate','input','MortgagePayment' if observer else 'MortgageTerms'),('outputs','observed' if observer else 'payment','output','MortgagePayment')]}}
    (out/'component.json').write_text(json.dumps(m,indent=2)+'\n')
    input_port='payment' if observer else 'calculate'
    output_port='observed' if observer else 'payment'
    semantic='MortgagePayment' if observer else 'MortgageTerms'
    capability='<https://workbench.example/vslice#ObserveMortgagePayment>' if observer else 'wb:CalculateMortgagePayment'
    (out/'capabilities.ttl').write_text(f'''@prefix wb: <{WB}> .
@prefix ex: <https://workbench.example/components/{name}/> .
ex:component a wb:Component ; wb:componentId "{identifier}" ; wb:componentVersion "0.1.0" ;
 wb:providesCapability {capability} ; wb:hasInputPort ex:calculate ; wb:hasOutputPort ex:payment ; wb:hasArtifact ex:artifact .
ex:calculate a wb:InputPort ; wb:portName "{input_port}" ; wb:schemaRef "schemas/mortgage-input.schema.json" ; wb:semanticType wb:{semantic} ; wb:unit wb:GBP .
ex:payment a wb:OutputPort ; wb:portName "{output_port}" ; wb:schemaRef "schemas/mortgage-output.schema.json" ; wb:semanticType wb:MortgagePayment ; wb:unit wb:GBP .
ex:artifact a wb:Artifact ; wb:artifactPath "logic.wasm" ; wb:sha256 "{digest}" .
''')
    hashes={str(p.relative_to(out)):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(out.rglob('*')) if p.is_file()}
    # Package hash listing stays OUTSIDE the strict bundle (no unreferenced files).
    (out.parent/f'{name}-files.json').write_text(json.dumps(hashes,indent=2)+'\n')
    print(f'{digest}  {out.relative_to(ROOT)}/logic.wasm')

if __name__=='__main__':
    package('mortgage-headless','mortgage_headless.wasm')
    package('mortgage-adversary','mortgage_adversary.wasm')

    package('mortgage-observer','mortgage_observer.wasm',observer=True)

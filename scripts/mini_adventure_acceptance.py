"""Check a complete native package, manual save/restart and exported mini adventure.

Runs offline without compiling. Writes evidence and isolated user data under target/.
"""
from pathlib import Path
import hashlib, json, os, shutil, subprocess, sys, tomllib, uuid, zipfile
repo=Path(__file__).resolve().parents[1]
version=tomllib.loads((repo/'Cargo.toml').read_text(encoding='utf-8'))['package']['version']
native='windows-x64' if os.name=='nt' else 'linux-x64'
package=(Path(sys.argv[1]) if len(sys.argv)>1 else repo/'dist'/f'Lupine-{version}-{native}').resolve(strict=True)
root=repo/'target'/('mini-package-acceptance-'+uuid.uuid4().hex[:8]);root.mkdir()
data=root/'userdata'
env=dict(os.environ,APPDATA=str(data),XDG_DATA_HOME=str(data),TERM='dumb')
exe=package/('lupine.exe' if os.name=='nt' else 'lupine')
game=package/'adventures/mini_adventure'
checks=[]
def check(ok,label):
    assert ok,label
    checks.append(label)
def run(args,commands,label,cwd=root):
    script=commands if isinstance(commands,str) else '\n'.join(commands)+'\n'
    result=subprocess.run([str(a) for a in args],input=script,text=True,encoding='utf-8',capture_output=True,cwd=cwd,env=env,timeout=90)
    (root/(label+'.txt')).write_text(result.stdout+result.stderr,encoding='utf-8')
    assert result.returncode==0,f'{label}: {result.stderr}'
    return result.stdout
def play(commands,label):return run([exe],['1','1',*commands],label)
def saved(slot):
    matches=list(data.rglob(slot+'.*.rork'));assert len(matches)==1,(slot,matches)
    return tomllib.loads(matches[0].read_text(encoding='utf-8'))
def progress(value):return {k:v for k,v in value.items() if k!='save_name'}
def snapshot(path):return {str(p.relative_to(path)):hashlib.sha256(p.read_bytes()).hexdigest() for p in path.rglob('*') if p.is_file()}
before=snapshot(package)
meta=tomllib.loads((package/'lupine.runtime.toml').read_text(encoding='utf-8'))
check(meta['target']==('windows-x64' if os.name=='nt' else 'linux-x64'),'native runtime target')
check(meta['sha256']==hashlib.sha256(exe.read_bytes()).hexdigest(),'runtime digest')
check(meta['version']==version,'runtime version')
check(not list(package.rglob('*.rork')),'distributed package has no player saves')
check(not (package/'PACKAGE_INCOMPLETE.txt').exists(),'package assembly completed')
version_output=run([exe,'--version'],'','version');check(version in version_output,'executable version')
menu=run([exe],['3','1','0','4'],'startup-help')
check('Lupine Mini Adventure' in menu and 'RPG Persistent Encounters Demonstration' not in menu,'focused player menu')
text=play(['talk guide','cancel','quit','4'],'cancel-name')
check('Conversation canceled' in text and 'Unsaved progress' not in text,'name cancellation and fresh quit are free')
check(not list(data.rglob('*.rork')),'cancellation does not save')
for child in [package/'story_example', *sorted((package/'samples').iterdir()), *sorted((package/'adventures').iterdir())]:
    if not (child/'game.toml').exists():continue
    check('Valid story:' in run([exe,'validate',child],'','validate-'+child.name),'validate '+child.name)
    if (child/'scenarios.toml').exists():
        out=run([exe,'test',child,child/'scenarios.toml'],'','scenarios-'+child.name)
        check('FAIL' not in out and 'PASS:' in out,'scenarios '+child.name)
check(not list(data.rglob('*.rork')),'all bundled scenarios leave saves untouched')
route=tomllib.loads((game/'scenarios.toml').read_text(encoding='utf-8'))['scenario'][0]['commands']
route=["Zoë O'Neil" if c=='Rex' else c for c in route]
first_attack=route.index('attack stone golem with steel sword')
preboss=route[:first_attack-1]
text=play(preboss+['save before-boss','quit','4'],'full-route-checkpoint')
checkpoint=saved('before-boss')
check(checkpoint['player_name']=="Zoë O'Neil",'Unicode character name saved')
check(checkpoint['current_room']=='approach' and checkpoint['wallet']['coins']==0,'complete route reaches approach with correct economy')
check('gate_open' in checkpoint['flags'] and checkpoint['objects']['containers']['chest']=={'open':True,'locked':False},'gate and chest progress saved')
check(checkpoint['shop_purchases']['trader:sword']==1,'trophy sale and sword purchase')
check(checkpoint['save_format_version']==7 and checkpoint['save_compatibility_version']==2,'RORK 7 and content revision 2')
check(not checkpoint['objects']['placements']['shop_key'] and not checkpoint['objects']['placements']['chest_key'],'shop and chest keys consumed only by unlocks')
text=play(['load before-boss','stats','inventory','save restored','quit','4'],'restart-restore')
check('Loaded: before-boss' in text and "Name: Zoë O'Neil" in text,'fresh process restores named character')
check(progress(checkpoint)==progress(saved('restored')),'all state round-trips across process restart')
text=play(['load before-boss','learn all','cast heling','jurnal','save invalid-inputs','quit','4'],'advisory-input')
check(progress(checkpoint)==progress(saved('invalid-inputs')),'unsupported bulk and typo suggestions do not mutate state')
text=play(['load before-boss','east','attack stone golem with steel sword','save wounded','west','west','use mana altar','east','east','save retained','quit','4'],'boss-retreat')
wounded=saved('wounded');retained=saved('retained')
check(wounded['active_enemies']==retained['active_enemies'],'boss wounds persist after retreat and return')
check(retained['character']['hp']==80 and retained['character']['mp']==30,'free altar restores health and mana')
text=play(['load wounded','eat ration','save fed','quit','4'],'consumable')
fed=saved('fed');check(fed['character']['hp']>wounded['character']['hp'],'ration restores player health')
count=lambda s:sum(x['quantity'] for x in s['objects']['placements']['ration'])
check(count(fed)==count(wounded)-1,'ration consumes exactly one unit')
text=play(['load before-boss','east',*route[first_attack:],'save victory','read coat','search coat','read journal','skills','wait','save after-reading','quit','4'],'victory')
victory=saved('victory');check(victory['events']['ending']=='victory' and 'Mini adventure complete!' in text,'stone golem defeated and success ending reached')
check(progress(victory)==progress(saved('after-reading')),'post-ending reading and rejected wait preserve state')
text=play(['load before-boss','save replay','quit','4'],'load-before-ending')
check(progress(checkpoint)==progress(saved('replay')),'prior checkpoint restores unended adventure')
# Exercise actual offer eligibility with the mini-adventure offers before buying.
market=route[:route.index('sell trophy')]
text=play(market+['save market-before','sell sword','buy trophy','save market-after','quit','4'],'wrong-trades')
check(progress(saved('market-before'))==progress(saved('market-after')),'wrong-direction trades preserve complete state')
save_hashes=snapshot(data)
# Export the actual mini adventure using the downloaded runtime and real creator UI.
copy=root/'Adventure copy with spaces';shutil.copytree(game,copy)
source_before=snapshot(copy)
destination=root/'Ready to share';number='1' if os.name=='nt' else '2'
text=run([exe,'creator',copy],f'e\n{number}\n1\n\n{destination}\nyes\n0\n','export-mini')
check('EXPORT COMPLETE!' in text,'mini adventure exported through creator')
check(snapshot(copy)==source_before,'export preserves authored files')
archive=next(destination.glob('*.zip'));extracted=root/'Extracted mini with spaces'
with zipfile.ZipFile(archive) as z:
    check(z.testzip() is None,'exported ZIP integrity')
    names=z.namelist();check(len(names)==len(set(names)) and all(not n.startswith('/') and '..' not in n.split('/') for n in names),'export paths are unique and confined')
    check(not any(n.endswith('.rork') for n in names),'export excludes player saves')
    z.extractall(extracted)
check((extracted/'THIRD_PARTY_NOTICES.txt').exists(),'export includes dependency notices')
check(snapshot(extracted/'adventure')=={k:v for k,v in source_before.items() if k!='scenarios.toml'},'export includes exact playable content (author scenarios intentionally excluded)')
launcher=[os.environ.get('COMSPEC','cmd.exe'),'/d','/c',str(extracted/'Play.cmd')] if os.name=='nt' else ['sh',str(extracted/'Play.sh')]
text=run(launcher,route+['quit','yes',''],'exported-launcher-full-victory')
check('Mini adventure complete!' in text,'exported launcher completes full adventure from unrelated directory')
check(snapshot(data)==save_hashes,'export and launcher do not rewrite or add saves')
check(snapshot(package)==before,'downloaded package remains byte-for-byte unchanged')
summary=dict(package=str(package),platform=meta['target'],version=meta['version'],runtime_sha256=meta['sha256'],passed=len(checks),checks=checks,explicit_save_slots=len(list(data.rglob('*.rork'))),automatic_saves=False,human_usability_test=False,evidence=str(root))
(root/'summary.json').write_text(json.dumps(summary,indent=2),encoding='utf-8')
print(json.dumps(summary,indent=2))

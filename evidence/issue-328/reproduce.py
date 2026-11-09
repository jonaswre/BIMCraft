from pathlib import Path
from PIL import Image
import hashlib, json, os, subprocess

ROOT = Path(__file__).resolve().parent
CLI = os.environ.get('BIMCRAFT_CLI', 'bimcraft-cli')

def call(name, args):
    result = subprocess.run([CLI, *args], cwd=ROOT, capture_output=True, text=True, encoding='utf-8')
    (ROOT / (name + '-cli.log')).write_text(result.stdout + result.stderr, encoding='utf-8')
    if result.returncode:
        raise SystemExit(result.stdout + result.stderr)

def run(stage):
    args = ['run']
    for line in (ROOT / 'reproduction.txt').read_text(encoding='utf-8-sig').splitlines():
        if line.strip():
            args += ['--cmd', line.replace('baseline.png', stage + '.png')]
    call(stage, args + ['--save', stage + '.bimcraft', '--save', stage + '.pdf'])
    image = Image.open(ROOT / (stage + '.png'))
    image.crop((1050, 1225, 2100, 1485)).save(ROOT / (stage + '-title-crop.png'))
    number = 'SYNTHETIC-A-2026-001-REVISION-A'
    command = 'render ' + json.dumps({'view': number, 'path': stage + '-reopened.png'})
    call(stage + '-reopened', ['run', stage + '.bimcraft', '--cmd', command, '--save', stage + '-reopened.pdf'])
    for ext in ['png', 'pdf']:
        assert (ROOT / (stage + '.' + ext)).read_bytes() == (ROOT / (stage + '-reopened.' + ext)).read_bytes(), ext
    call(stage + '-short', ['run', '--cmd', 'new {"name":"Project"}', '--cmd', 'sheet {"number":"A101","name":"Plan","size":"A3"}', '--cmd', 'render ' + json.dumps({'view': 'A101', 'path': stage + '-short.png'}), '--save', stage + '-short.pdf'])
    if stage == 'fixed':
        for ext in ['png', 'pdf']:
            assert (ROOT / ('baseline-short.' + ext)).read_bytes() == (ROOT / ('fixed-short.' + ext)).read_bytes(), 'short ' + ext
    summary = {'stage': stage, 'size': image.size, 'native_reopen_png_pdf_identical': True, 'short_values_unchanged': stage == 'fixed', 'png_sha256': hashlib.sha256((ROOT / (stage + '.png')).read_bytes()).hexdigest()}
    (ROOT / (stage + '-journey.json')).write_text(json.dumps(summary, indent=2), encoding='utf-8')
    print(json.dumps(summary))

if __name__ == '__main__':
    import sys
    run(sys.argv[1])

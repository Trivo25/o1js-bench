import { mkdirSync, writeFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';

function getArg(name, fallback) {
  const idx = process.argv.indexOf(name);
  return idx === -1 ? fallback : process.argv[idx + 1];
}

const verbose = process.argv.includes('--verbose');
const iterations = Number(getArg('--iterations', '3'));

const tools = [
  {
    name: 'o1js',
    command: [
      'npm', '--prefix', 'o1js', 'run', 'bench', '--',
      '--iterations', String(iterations),
      ...(verbose ? [] : ['--json'])
    ]
  },
  {
    name: 'jolt',
    command: [
      'cargo', 'run', '--release', '-p', 'jolt-ecdsa-bench', '--',
      '--iterations', String(iterations),
      ...(verbose ? [] : ['--json'])
    ]
  }
];

function extractJson(stdout, name) {
  const lines = stdout.trim().split(/\r?\n/).reverse();
  for (const line of lines) {
    const trimmed = line.trim();
    if (!trimmed.startsWith('{')) continue;
    try { return JSON.parse(trimmed); } catch {}
  }
  throw new Error(`no JSON result found in ${name} output`);
}

const startedAt = new Date().toISOString();
const results = [];
for (const tool of tools) {
  console.error(`\n== ${tool.name} ==`);
  console.error(`$ ${tool.command.join(' ')}`);
  const run = spawnSync(tool.command[0], tool.command.slice(1), {
    cwd: new URL('..', import.meta.url),
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
    env: { ...process.env, RISC0_DEV_MODE: '0' }
  });
  process.stderr.write(run.stderr);
  process.stderr.write(run.stdout);
  if (run.status !== 0) throw new Error(`${tool.name} benchmark failed with status ${run.status}`);
  results.push(extractJson(run.stdout, tool.name));
}

const report = { startedAt, iterations, fixture: 'shared/fixtures/ecdsa_secp256k1.json', results };
mkdirSync('results', { recursive: true });
writeFileSync('results/latest.json', `${JSON.stringify(report, null, 2)}\n`);
const rows = [
  '| tool | iterations | min ms | median ms | max ms | samples ms |',
  '| --- | ---: | ---: | ---: | ---: | --- |',
  ...results.map((r) => `| ${r.tool} | ${r.iterations} | ${r.minMs.toFixed(3)} | ${r.medianMs.toFixed(3)} | ${r.maxMs.toFixed(3)} | ${r.samplesMs.map((x) => x.toFixed(3)).join(', ')} |`)
];
writeFileSync('results/latest.md', `# ECDSA benchmark results\n\n${rows.join('\n')}\n`);
console.log(JSON.stringify(report));

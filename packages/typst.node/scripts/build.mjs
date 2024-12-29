import { spawnSync } from 'child_process';

const args = process.argv.slice(2);

const napiArgs = [
  'napi',
  'build',
  '--platform',
  '--release',
  '--dts-header',
  "\"/* auto-generated by NAPI-RS */ /* eslint-disable */ import type * as types from './index.js';\"",
  '--dts',
  'index-napi.d.ts',
  '--js',
  'index-napi.js',
  '--pipe',
  '\"prettier -w\"',
  ...args,
];

const YARN = process.platform === 'win32' ? 'yarn.cmd' : 'yarn';

// https://github.com/nodejs/node/issues/52554
const opts = { stdio: 'inherit', shell: true };

const napi = spawnSync(YARN, napiArgs, opts);
if (napi.error || napi.status !== 0) {
  console.log('napi', napi.status, napi.error);
  process.exit(napi.status);
}

const tsc = spawnSync(YARN, ['tsc'], opts);
if (tsc.error || tsc.status !== 0) {
  console.log('tsc', tsc.status, tsc.error);
  process.exit(tsc.status);
}

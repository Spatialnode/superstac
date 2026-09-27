const [major, minor] = process.versions.node.split('.').map(Number);
if (major < 22 || (major === 22 && minor < 19) || !process.features.require_module) {
  console.error(
    `SuperSTAC docs require Node.js >=22.19.0 with require(ESM) enabled; you are running ${process.version}.\n` +
    'From docs/, run "nvm install && nvm use", then restart the dev server.\n' +
    'If your Node version is supported, remove --no-experimental-require-module from NODE_OPTIONS.',
  );
  process.exit(1);
}

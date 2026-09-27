export function Logo() {
  return (
    <span className="brand-logo">
      {/* Plain SVG images preserve the exact artwork and need no image optimizer. */}
      <img src="/superstac/brand/superstac-logo.svg" alt="SuperSTAC" width={1397} height={470} className="logo-light" />
      <img src="/superstac/brand/superstac-logo-dark.svg" alt="SuperSTAC" width={1397} height={470} className="logo-dark" />
    </span>
  );
}

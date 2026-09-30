export function Progress({ pct }: { pct: number }) {
  return (
    <div className="ui-progress" role="progressbar" aria-valuenow={pct} aria-valuemin={0} aria-valuemax={100}>
      <div style={{ width: pct + "%" }} />
    </div>
  );
}

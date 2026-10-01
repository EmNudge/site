import type { ReactNode } from "react";

export interface Column<T> {
  key: string;
  header: ReactNode;
  align?: "right";
  /** Dropped on narrow screens, where the table would otherwise have to scroll. */
  secondary?: boolean;
  render?: (row: T) => ReactNode;
}

const cellClass = <T,>(c: Column<T>) =>
  [c.align === "right" && "num", c.secondary && "secondary"].filter(Boolean).join(" ") || undefined;

export function DataTable<T>({
  columns,
  rows,
  empty = "Nothing here yet.",
  error,
  getKey,
}: {
  columns: Column<T>[];
  rows: T[];
  empty?: ReactNode;
  error?: string;
  getKey?: (row: T, i: number) => string | number;
}) {
  return (
    <div className="ui-table-wrap">
      <table className="ui-table">
        <thead>
          <tr>
            {columns.map((c) => (
              <th key={c.key} className={cellClass(c)}>
                {c.header}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {error ? (
            <tr>
              <td colSpan={columns.length} className="ui-table__empty" style={{ color: "var(--danger)" }}>
                {error}
              </td>
            </tr>
          ) : rows.length === 0 ? (
            <tr>
              <td colSpan={columns.length} className="ui-table__empty">
                {empty}
              </td>
            </tr>
          ) : (
            rows.map((row, i) => (
              <tr key={getKey ? getKey(row, i) : i}>
                {columns.map((c) => (
                  <td key={c.key} className={cellClass(c)}>
                    {c.render ? c.render(row) : (row as Record<string, ReactNode>)[c.key]}
                  </td>
                ))}
              </tr>
            ))
          )}
        </tbody>
      </table>
    </div>
  );
}

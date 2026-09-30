import type { ReactNode } from "react";

export function Section({
  title,
  desc,
  action,
  children,
}: {
  title: ReactNode;
  desc?: ReactNode;
  action?: ReactNode;
  children: ReactNode;
}) {
  return (
    <section className="ui-section">
      <div className="ui-section__head">
        <div>
          <h2 className="ui-section__title">{title}</h2>
          {desc != null && <p className="ui-section__desc">{desc}</p>}
        </div>
        {action}
      </div>
      {children}
    </section>
  );
}

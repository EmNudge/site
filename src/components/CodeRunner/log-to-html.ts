const REGEX = () => /%([oOdisfc])|.\d+(f)/g;

const stringify = (data: any) => {
  const type = typeof data;

  if (type === "bigint") return `${type}n`;
  if (!["object", "function"].includes(type)) return String(data);

  if (data[Symbol.toStringTag]) return data[Symbol.toStringTag];

  const jsonStr = JSON.stringify(data);
  if (jsonStr) return jsonStr;

  if (typeof data.toString === "function") {
    return data.toString();
  }

  return String(data);
};

const el = (name = "span", content = "", props = {}) => {
  const el = document.createElement(name);
  el.textContent = content;
  for (const prop in props) el[prop] = props[prop];

  return el;
};

const colorize = (data: any) => {
  const type = typeof data;
  if (type === "string") {
    return el("span", `"${data}"`, { style: "color: #ce9178" });
  }
  const text = stringify(data);
  if (!["object", "function"].includes(type)) {
    return el("span", text, { style: "color: #c586c0" });
  }
  return el("span", text);
};

export function logToHtml(...args: any[]) {
  const dom = document.createElement("console-log");
  const initial = args[0];

  // track %c
  let styledEl: HTMLElement | null = null;
  const append = (content: string, props = {}): HTMLElement =>
    (styledEl || dom).appendChild(el("span", content, props));

  if (typeof initial !== "string" || !REGEX().test(initial)) {
    for (const thing of args) {
      if (typeof thing === "number") {
        dom.appendChild(colorize(thing));
      } else {
        append(stringify(thing));
      }

      append("", { className: "spacer" });
    }

    return dom;
  }

  const matches = initial.matchAll(REGEX());
  let lastIndex = 0;
  let index = 1;
  for (const match of matches) {
    append(initial.slice(lastIndex, match.index));

    const arg = args[index];
    const symbol = match[1] || match[2];

    if (/[dif]/.test(symbol)) {
      append(String(Number(arg)));
    } else if (/[sO]/.test(symbol)) {
      append(stringify(arg));
    } else if (symbol === "c") {
      styledEl = append("", { style: arg });
    } else {
      // %o
      dom.appendChild(colorize(arg));
    }

    lastIndex = match.index + 2;
    index++;
  }

  append(initial.slice(lastIndex));
  styledEl = null;

  for (let i = index; i < args.length; i++) {
    append("", { className: "spacer" });
    append(stringify(args[i]));
  }

  return dom;
}

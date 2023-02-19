import { logToHtml } from './log-to-html';

// code to execute on a web worker for purposes of eval
const WORKER_CODE = `
const console = new Proxy({}, {
  get(_target, key) {
    return (...args) => {
      postMessage({ type: key, args });
    }
  }
});
self.addEventListener('error', e => {
  postMessage({ type: 'error', args: [e.message] });
  e.preventDefault();
});
`

interface ElementWorker {
  worker: Worker;
  terminatorId: number;
}
const workerMap = new WeakMap<Element, ElementWorker>();

const onExecute = (codeBlock: Element) => {
  if (workerMap.has(codeBlock)) {
    const { worker, terminatorId } = workerMap.get(codeBlock);

    if (worker) {
      clearTimeout(terminatorId);
      worker.terminate();
    }
  }

  const outputEl = codeBlock.querySelector('output')
  outputEl.innerHTML = '';
  
  let worker: Worker;
  try {
    const code = codeBlock.querySelector('code').textContent;
    const blob = new Blob([WORKER_CODE, code], { type: 'application/javascript' });
    const url = URL.createObjectURL(blob);
    worker = new Worker(url);
  } catch (e) {
    console.error(e);
  }

  const addLog = (type: string, ...args: any[]) => {
    const logEl = logToHtml(...args);
    logEl.setAttribute('data-type', type.toLowerCase());
    logEl.style.setProperty('--index', String(outputEl.children.length));

    outputEl.appendChild(logEl);
    outputEl.classList.add('opened');

    console[type](...args);
  }

  worker.addEventListener('message', ({ data }) => addLog(data.type, ...data.args));
  worker.addEventListener('error', (e) => addLog('error', e.message));

  const terminatorId = setTimeout(() => {
    worker.terminate();
  }, 5000) as unknown as number;

  workerMap.set(codeBlock, { terminatorId, worker });
}

const h = (name: string, attributes: Record<string, string> | null = null, ...children: (string | Node)[]) => {
  const el = document.createElement(name);
  if (attributes) Object.assign(el, attributes);
  el.append(...children);
  return el;
}

export function addRunnableCode() {
  const codeBlocks = document.querySelectorAll('[data-rehype-pretty-code-fragment]');
  const runnableCodeBlocks = [...codeBlocks].filter(codeBlock => {
    const titleEl = codeBlock.querySelector('[data-rehype-pretty-code-title]');
    if (!titleEl || !('dataset' in titleEl)) return false;
    
    const lang = (titleEl.dataset as any)?.language
    if (!['js', 'javascript'].includes(lang)) return false;
    return titleEl.textContent.endsWith('(runnable)');
  });

  for (const codeBlock of runnableCodeBlocks) {
    codeBlock.classList.add('runnable')

    const titleEl = codeBlock.querySelector('[data-rehype-pretty-code-title]');

    const newTitleEl = h('span', null, titleEl.textContent);
    const runButton = h('button', null, 'Run');
    const clearButton = h('button', null, 'Clear');
    titleEl.textContent = '';
    titleEl.append(newTitleEl, runButton, clearButton);

    runButton.addEventListener('click', () => onExecute(codeBlock));
    clearButton.addEventListener('click', () => {
      const outputEl = codeBlock.querySelector('output');
      if (outputEl && outputEl.classList.contains('opened')) {
        outputEl.classList.remove('opened');
        outputEl.textContent = '';
      }
    });

    codeBlock.append(h('output'));
  }
}
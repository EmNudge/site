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

export function addRunnableCode() {
  if (customElements.get('runnable-code')) return;

  class RunnableCode extends HTMLPreElement {
    code = '';
    outputEl: HTMLElement | null = null;
  
    worker: Worker | null = null;
    terminatorId: number | null = null;
  
    constructor() {
      super();
      console.log(this.localName)
  
      this.code = this.querySelector('code').textContent;
  
      const runButton = document.createElement('button');
      runButton.textContent = 'Run'; // '▶'
      this.prepend(runButton);
  
      this.outputEl = document.createElement('output');
      this.appendChild(this.outputEl);
  
      this.onExecute = this.onExecute.bind(this);
      runButton.addEventListener('click', this.onExecute);
    }
  
    onExecute() {
      if (this.worker) {
        clearTimeout(this.terminatorId);
        this.worker.terminate();
      }
  
      this.outputEl.innerHTML = '';
      this.outputEl.className = '';
  
      const blob = new Blob([WORKER_CODE, this.code], { type: 'application/javascript' });
      const url = URL.createObjectURL(blob);
      this.worker = new Worker(url);
  
      this.worker.addEventListener('message', (e) => {
        this.addLog(e.data);
        console[e.data.type](...e.data.args);
      });
  
      this.terminatorId = setTimeout(() => {
        this.worker.terminate();
      }, 5000) as unknown as number;
    }
  
    addLog({ type, args }) {
      const logEl = logToHtml(...args);
      logEl.setAttribute('data-type', type.toLowerCase());
  
      this.outputEl.appendChild(logEl);
      this.outputEl.classList.add('opened');
    }
  }
  
  customElements.define('runnable-code', RunnableCode, { extends: 'pre' });
}

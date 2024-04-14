import { definePlugin } from '@expressive-code/core'
import { addClassName, select, h } from '@expressive-code/core/hast'

export function codeRunnerPlugin() {
  return definePlugin({
    name: 'Add ability to run JS code',
    hooks: {
      postprocessRenderedBlock: ({ codeBlock, renderData }) => {
        // Only apply this to code blocks with the `error-preview` meta
        if (!codeBlock.meta.includes('runnable')) return
        
        // @ts-ignore
        const header = select('.header', renderData.blockAst);
        header.children.push(h('button', { className: 'runnable-btn' }, 'Run Code'));
      },
    },
  })
}
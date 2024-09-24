import React from 'react'
import ReactMarkdown from 'react-markdown'
import remarkMath from 'remark-math'
import remarkGfm from 'remark-gfm'
import remarkBreaks from 'remark-breaks';
import rehypeKatex from 'rehype-katex'
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter'

const RenderMarkdown = ({ markdown }) => {
  return React.createElement(ReactMarkdown, {
    remarkPlugins: [remarkMath, remarkGfm, remarkBreaks],
    rehypePlugins: [rehypeKatex],
    children: markdown,
    components: {
      code({ node, inline, className = '', children, ...props }) {
        const language = /language-(\w+)/.exec(className);
        const showLineNumbers = /=\d*$/.test(className);
        const numbersFrom = +/\d+$/.exec(className)?.[0] || 1;
        return !inline
          ? React.createElement(SyntaxHighlighter, {
            ...props,
            children: String(children).replace(/\n$/, ''),
            language: language?.[1] || 'text',
            showLineNumbers: showLineNumbers,
            startingLineNumber: numbersFrom,
            PreTag: 'div',
            customStyle: {padding: '2px'},
            lineNumberStyle:{
              borderRight: '3px solid #b9b9b9',
              marginRight: '8px',
              paddingRight: '5px',
            },
          })
          : React.createElement('code', { ...props, className }, children)
      },
    },
  })
}

export default RenderMarkdown

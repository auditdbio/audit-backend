import React from 'react'
import ReactMarkdown from 'react-markdown'
import remarkMath from 'remark-math'
import remarkGfm from 'remark-gfm'
import rehypeKatex from 'rehype-katex'
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter'

const RenderMarkdown = ({ markdown }) => {
  const markdownForPrint = markdown?.replace(/(?<!!)\[[^\]]+]\((https?:\/\/[^)]+)\)/g, '$1')

  return React.createElement(ReactMarkdown, {
    remarkPlugins: [remarkMath, remarkGfm],
    rehypePlugins: [rehypeKatex],
    children: markdownForPrint,
    components: {
      code({ node, inline, className, children, ...props }) {
        const match = /language-(\w+)/.exec(className || '')
        return !inline && match
          ? React.createElement(SyntaxHighlighter, {
              ...props,
              children: String(children).replace(/\n$/, ''),
              language: match[1],
              PreTag: 'div',
            })
          : React.createElement('code', { ...props, className }, children)
      },
    },
  })
}

export default RenderMarkdown

const linkShortener = link => {
  return link
    .replace(/(^https?:\/\/)|(github\.com\/)|(raw\.githubusercontent\.com\/)/gi, '')
    .replace(/(?<=\/)blob\/(?=[0-9a-f]{40}\/)/i, '')
    .replace(/(?<=\/)[0-9a-f]{40}(?=\/)/gi, m => m.slice(0, 5) + '...')
}

export default linkShortener

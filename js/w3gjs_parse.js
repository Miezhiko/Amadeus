const W3GReplay = require('w3gjs').default
const parser = new W3GReplay();

( async () => {
  // node script argv[2]
  const result = await parser.parse(process.argv[2])
  console.log( JSON.stringify(result, null, '  ') )
})().catch(console.error);

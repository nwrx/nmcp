import { Client } from '@modelcontextprotocol/sdk/client/index.js'
import { SSEClientTransport } from '@modelcontextprotocol/sdk/client/sse.js'
import { argv } from 'node:process'

async function main() {
  const client = new Client({
    name: 'example-client',
    version: '0.0.1',
  })

  const args = argv.slice(2)
  const url = new URL(args[0])
  const transport = new SSEClientTransport(url)
  await client.connect(transport)

  console.log(`Connected to ${url.toString()}`)
  const tools =  await client.listTools()
  console.log(`Available tools: ${tools.tools.map(tool => tool.name).join(', ')}`)
}

main()
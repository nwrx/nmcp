/* eslint-disable unicorn/prefer-top-level-await */
import { Client } from '@modelcontextprotocol/sdk/client/index.js'
import { SSEClientTransport } from '@modelcontextprotocol/sdk/client/sse.js'
import consola from 'consola'

consola.wrapAll()

async function main() {
  const client = new Client({
    name: 'example-client',
    version: '0.0.1',
  })

  const args = process.argv.slice(2)
  const url = new URL(args[0])
  const transport = new SSEClientTransport(url)
  await client.connect(transport)
  consola.log(`Connected to ${url.toString()}`)

  const tools = await client.listTools()
  consola.log(`Available tools: ${tools.tools.map(tool => tool.name).join(', ')}`)
}

void main()

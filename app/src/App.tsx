import { useState } from 'react'

function App() {
  const [version] = useState('0.1.0')

  return (
    <div className="min-h-screen bg-background p-8">
      <h1 className="text-2xl font-bold">AsioBridge {version}</h1>
      <p className="mt-2 text-muted-foreground">
        Modern replacement for ASIO Link Pro
      </p>
    </div>
  )
}

export default App

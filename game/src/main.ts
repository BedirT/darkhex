import { BoardScene } from './scenes/BoardScene'

const app = document.getElementById('app')!
const scene = new BoardScene(app)
scene.init()

// Clean up on Vite HMR to prevent leaked listeners / RAF loops.
if (import.meta.hot) {
  import.meta.hot.dispose(() => scene.dispose())
}

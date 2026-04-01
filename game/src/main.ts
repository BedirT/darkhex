import { BoardScene } from './scenes/BoardScene'

const app = document.getElementById('app')!
const scene = new BoardScene(app)
scene.init()

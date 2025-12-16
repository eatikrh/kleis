import { createRoot } from 'react-dom/client'

// PatternFly CSS (must be imported before App)
import '@patternfly/patternfly/patternfly.css'
import '@patternfly/patternfly/patternfly-addons.css'

import App from './App.tsx'
import './index.css'

createRoot(document.getElementById('root')!).render(
  <App />,
)

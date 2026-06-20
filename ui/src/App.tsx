import { Routes, Route } from 'react-router-dom';
import { Layout } from './components/layout';
import RoutingRulesPage from './pages/routing-rules';
import TopicValidationsPage from './pages/topic-validations';
import PlaygroundPage from './pages/playground';
import { Toaster } from "@/components/ui/toaster"

function App() {
  return (
    <Layout>
      <Routes>
        <Route path="/" element={<RoutingRulesPage />} />
        <Route path="/routing-rules" element={<RoutingRulesPage />} />
        <Route path="/topic-validations" element={<TopicValidationsPage />} />
        <Route path="/playground" element={<PlaygroundPage />} />
      </Routes>
      <Toaster />
    </Layout>
  );
}

export default App;

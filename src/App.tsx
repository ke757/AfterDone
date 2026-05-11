import { useEffect } from 'react';
import { useUIStore, initAllListeners, cleanupAllListeners, useGoalStore, useAgentStore } from './stores';
import { Layout } from './layout/Layout';
import { TooltipProvider } from '@/components/ui/tooltip';
import { Toaster } from 'sonner';

function App() {
  const fetchGoals = useGoalStore((state) => state.fetchGoals);
  const fetchStatus = useAgentStore((state) => state.fetchStatus);
  const initRepo = useUIStore((state) => state.initRepo);

  useEffect(() => {
    initAllListeners();
    fetchGoals();
    fetchStatus();
    initRepo();

    return () => {
      cleanupAllListeners();
    };
  }, [fetchGoals, fetchStatus, initRepo]);

  return (
    <TooltipProvider>
      <Layout />
      <Toaster position="bottom-right" />
    </TooltipProvider>
  );
}

export default App;
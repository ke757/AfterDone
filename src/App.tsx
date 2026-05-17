import { useEffect } from 'react';
import { useUIStore, initAllListeners, cleanupAllListeners } from './stores';
import { Layout } from './layout/Layout';
import { TooltipProvider } from '@/components/ui/tooltip';
import { Toaster } from 'sonner';

function App() {
  const launchApp = useUIStore((state) => state.launchApp);
  const initStatus = useUIStore((state) => state.initStatus);

  useEffect(() => {
    launchApp();
  }, [launchApp]);

  useEffect(() => {
    if (initStatus === 'ready') {
      initAllListeners();
    }
    return () => {
      cleanupAllListeners();
    };
  }, [initStatus]);

  return (
    <TooltipProvider>
      <Layout />
      <Toaster position="bottom-right" />
    </TooltipProvider>
  );
}

export default App;
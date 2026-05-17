import { useEffect } from 'react';
import { useUIStore, initAllListeners, cleanupAllListeners } from './stores';
import { Layout } from './layout/Layout';
import { TooltipProvider } from '@/components/ui/tooltip';
import { Toaster } from 'sonner';

function App() {
  const initApp = useUIStore((state) => state.initApp);
  const initStatus = useUIStore((state) => state.initStatus);

  useEffect(() => {
    initApp();
  }, [initApp]);

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
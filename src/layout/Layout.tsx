import { useUIStore } from '@/stores/uiStore';
import { Header } from './Header';
import { LeftSidebar } from './LeftSidebar';
import { RightSidebar } from './RightSidebar';
import { WelcomePage } from '@/features/welcome/WelcomePage';
import { QuickStartPage } from '@/features/welcome/QuickStartPage';
import { SettingsPage } from '@/features/settings/SettingsPage';
import { AboutPage } from '@/features/about/AboutPage';

export function Layout() {
  const leftSidebarVisible = useUIStore((s) => s.leftSidebarVisible);
  const rightSidebarVisible = useUIStore((s) => s.rightSidebarVisible);
  const activePage = useUIStore((s) => s.activePage);
  const initStatus = useUIStore((s) => s.initStatus);

  if (initStatus === 'loading') {
    return (
      <div className="flex h-screen items-center justify-center bg-background text-foreground">
        <p className="text-sm text-muted-foreground">正在加载...</p>
      </div>
    );
  }

  if (initStatus === 'initialize') {
    return (
      <div className="flex h-screen flex-col bg-background text-foreground">
        <QuickStartPage />
      </div>
    );
  }

  return (
    <div className="flex h-screen flex-col bg-background text-foreground">
      <Header />
      <div className="flex min-h-0 flex-1 overflow-hidden">
        {leftSidebarVisible && <LeftSidebar />}
        <main className="flex-1 overflow-auto">
          {activePage === 'workspace' ? (
            <div className="flex h-full items-center justify-center text-muted-foreground">
              工作空间内容（开发中）
            </div>
          ) : activePage === 'settings' ? (
            <SettingsPage />
          ) : activePage === 'about' ? (
            <AboutPage />
          ) : (
            <WelcomePage />
          )}
        </main>
        {rightSidebarVisible && <RightSidebar />}
      </div>
    </div>
  );
}
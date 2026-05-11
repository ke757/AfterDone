import { useUIStore } from '@/stores/uiStore';
import { Header } from './Header';
import { LeftSidebar } from './LeftSidebar';
import { RightSidebar } from './RightSidebar';
import { WelcomePage } from '@/features/welcome/WelcomePage';
import { SettingsPage } from '@/features/settings/SettingsPage';
import { AboutPage } from '@/features/about/AboutPage';

export function Layout() {
  const leftSidebarVisible = useUIStore((s) => s.leftSidebarVisible);
  const rightSidebarVisible = useUIStore((s) => s.rightSidebarVisible);
  const activePage = useUIStore((s) => s.activePage);

  const renderMain = () => {
    switch (activePage) {
      case 'welcome':
        return <WelcomePage />;
      case 'workspace':
        return (
          <div className="flex h-full items-center justify-center text-muted-foreground">
            工作空间内容（开发中）
          </div>
        );
      case 'settings':
        return <SettingsPage />;
      case 'about':
        return <AboutPage />;
      case 'repo':
        return <WelcomePage />;
      default:
        return <WelcomePage />;
    }
  };

  return (
    <div className="flex h-screen flex-col bg-background text-foreground">
      <Header />
      <div className="flex min-h-0 flex-1 overflow-hidden">
        {leftSidebarVisible && <LeftSidebar />}
        <main className="flex-1 overflow-auto">{renderMain()}</main>
        {rightSidebarVisible && <RightSidebar />}
      </div>
    </div>
  );
}
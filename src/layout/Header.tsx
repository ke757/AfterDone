import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Button } from '@/components/ui/button';
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import {
  ChevronDown,
  PanelLeft,
  PanelRight,
  Settings,
  FolderOpen,
  Info,
} from 'lucide-react';
import { useUIStore } from '@/stores/uiStore';

export function Header() {
  const repoName = useUIStore((s) => s.currentRepoName);
  const toggleLeftSidebar = useUIStore((s) => s.toggleLeftSidebar);
  const toggleRightSidebar = useUIStore((s) => s.toggleRightSidebar);
  const setActivePage = useUIStore((s) => s.setActivePage);
  const selectRepo = useUIStore((s) => s.selectRepo);

  return (
    <header className="flex h-10 shrink-0 items-center justify-between border-b border-border bg-background px-2">
      <div className="flex items-center gap-1">
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="ghost" size="default" className="gap-1 font-medium">
              <FolderOpen className="size-4" />
              <span className="max-w-[200px] truncate">
                {repoName ?? '未选择仓库'}
              </span>
              <ChevronDown className="size-3" />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="start">
            <DropdownMenuItem
              onClick={() => setActivePage('settings')}
            >
              <Settings className="mr-2 size-4" />
              设置
            </DropdownMenuItem>
            <DropdownMenuItem
              onClick={() => selectRepo()}
            >
              <FolderOpen className="mr-2 size-4" />
              数据仓库
            </DropdownMenuItem>
            <DropdownMenuItem
              onClick={() => setActivePage('about')}
            >
              <Info className="mr-2 size-4" />
              关于
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>

        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="ghost"
              size="icon"
              onClick={toggleLeftSidebar}
            >
              <PanelLeft className="size-4" />
            </Button>
          </TooltipTrigger>
          <TooltipContent>切换左栏</TooltipContent>
        </Tooltip>
      </div>

      <div className="flex items-center gap-1">
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="ghost"
              size="icon"
              onClick={toggleRightSidebar}
            >
              <PanelRight className="size-4" />
            </Button>
          </TooltipTrigger>
          <TooltipContent>切换右栏</TooltipContent>
        </Tooltip>
      </div>
    </header>
  );
}
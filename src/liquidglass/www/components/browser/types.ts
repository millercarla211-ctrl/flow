export interface SVGLogo {
  id: number;
  title: string;
  component: React.ComponentType<React.SVGProps<SVGSVGElement>>;
}

export interface Tab {
  id: string;
  title: string;
  url: string;
  favicon?: string;
  pinned?: boolean;
  workspaceId: string;
  folderId?: string | null;
}

export interface TabFolder {
  id: string;
  name: string;
  collapsed: boolean;
  tabs: Tab[];
  workspaceId: string;
  parentId?: string | null;
}

export interface Workspace {
  id: string;
  name: string;
  color: string;
  icon: {
    type: "emoji" | "icon" | "dot";
    value: string;
  };
}

export type DropPosition = "before" | "after" | "inside" | null;

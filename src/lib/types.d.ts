export interface Thumbnail {
  url: string;
  width: number;
  height: number;
}

export interface Video { 
  title: string;
  id: string;
  thumbnails: { default: Thumbnail, medium: Thumbnail, high: Thumbnail }
}

export interface BlogPostMeta {
  title: string;
  slug: string;
  summary: string;
  date: string;
  minuteLength: number;
}

export interface BlogPost extends BlogPostMeta {
  html: string;
}


export interface Read {
  author: string;
  link: string;
  description: string;
  title: string;
  readAt: string;
  createdAt: string;
}
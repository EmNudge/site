export const STANDARD_SITE_DID = "did:plc:tgatoi47bb7xrxexwk7ogx73";

export const PUBLICATIONS = {
  blog: {
    rkey: "blog",
    url: "https://emnudge.dev/blog",
    name: "EmNudge Blog",
    description: "Long-form articles on software, language, and the web.",
  },
  notes: {
    rkey: "notes",
    url: "https://emnudge.dev/notes",
    name: "EmNudge Notes",
    description: "Short notes and quick thoughts.",
  },
} as const;

export type PublicationKind = keyof typeof PUBLICATIONS;

export const getDocumentRkey = (kind: PublicationKind, slug: string) => `${kind}-${slug}`;

export const getPublicationAtUri = (kind: PublicationKind) =>
  `at://${STANDARD_SITE_DID}/site.standard.publication/${PUBLICATIONS[kind].rkey}`;

export const getDocumentAtUri = (kind: PublicationKind, slug: string) =>
  `at://${STANDARD_SITE_DID}/site.standard.document/${getDocumentRkey(kind, slug)}`;

export const getKindFromUrl = (url: string): PublicationKind | null => {
  if (url.startsWith("/blog/")) return "blog";
  if (url.startsWith("/notes/")) return "notes";
  return null;
};

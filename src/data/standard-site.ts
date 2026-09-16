import rkeys from "../../scripts/standard-site-rkeys.json";

export const STANDARD_SITE_DID = "did:plc:tgatoi47bb7xrxexwk7ogx73";

export const PUBLICATIONS = {
  blog: {
    url: "https://emnudge.dev/blog",
    name: "EmNudge Blog",
    description: "Long-form articles on software, language, and the web.",
  },
  notes: {
    url: "https://emnudge.dev/notes",
    name: "EmNudge Notes",
    description: "Short notes and quick thoughts.",
  },
} as const;

export type PublicationKind = keyof typeof PUBLICATIONS;

// standard.site lexicons declare `key: tid`, so record keys must be TIDs, not
// human-readable slugs. The slug -> TID mapping lives in
// scripts/standard-site-rkeys.json and is maintained by scripts/sync-standard-site.mjs.
const publicationRkeys = rkeys.publications as Record<string, string>;
const documentRkeys = rkeys.documents as Record<string, string>;

export const getDocumentMapKey = (kind: PublicationKind, slug: string) => `${kind}/${slug}`;

export const getPublicationRkey = (kind: PublicationKind): string | undefined =>
  publicationRkeys[kind];

export const getDocumentRkey = (kind: PublicationKind, slug: string): string | undefined =>
  documentRkeys[getDocumentMapKey(kind, slug)];

export const getPublicationAtUri = (kind: PublicationKind): string | null => {
  const rkey = getPublicationRkey(kind);
  return rkey ? `at://${STANDARD_SITE_DID}/site.standard.publication/${rkey}` : null;
};

export const getDocumentAtUri = (kind: PublicationKind, slug: string): string | null => {
  const rkey = getDocumentRkey(kind, slug);
  return rkey ? `at://${STANDARD_SITE_DID}/site.standard.document/${rkey}` : null;
};

export const getKindFromUrl = (url: string): PublicationKind | null => {
  if (url.startsWith("/blog/")) return "blog";
  if (url.startsWith("/notes/")) return "notes";
  return null;
};

module SlopeRoute

%default total

public export
data Route = Denied | Kernel | Service | CosmicSession

public export
record Evidence where
  constructor MkEvidence
  authority : Bool
  live : Bool
  bounded : Bool
  cosmicReady : Bool

public export
selectRoute : Evidence -> Route
selectRoute evidence =
  if not evidence.authority || not evidence.bounded then Denied
  else if evidence.cosmicReady && evidence.live then CosmicSession
  else if evidence.live then Service
  else Kernel

public export
data Admitted : Route -> Type where
  KernelRoute : Admitted Kernel
  ServiceRoute : Admitted Service
  CosmicRoute : Admitted CosmicSession

public export
admit : (route : Route) -> Maybe (Admitted route)
admit Denied = Nothing
admit Kernel = Just KernelRoute
admit Service = Just ServiceRoute
admit CosmicSession = Just CosmicRoute

public export
resolveRoute : (evidence : Evidence) -> Maybe (route ** Admitted route)
resolveRoute evidence =
  let selected = selectRoute evidence in
  case admit selected of
    Nothing => Nothing
    Just witness => Just (selected ** witness)

public export
cosmicExample : Maybe (route ** Admitted route)
cosmicExample = resolveRoute (MkEvidence True True True True)

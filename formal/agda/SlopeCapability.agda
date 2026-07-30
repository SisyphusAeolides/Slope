{-# OPTIONS --safe --without-K #-}

module SlopeCapability where

data Empty : Set where

Not : Set -> Set
Not proposition = proposition -> Empty

data Route : Set where
  denied kernel service cosmicSession : Route

data Capability : Route -> Set where
  kernelCapability : Capability kernel
  serviceCapability : Capability service
  cosmicCapability : Capability cosmicSession

deniedHasNoCapability : Not (Capability denied)
deniedHasNoCapability ()

data Transition : Route -> Route -> Set where
  kernelToService : Transition kernel service
  serviceToCosmic : Transition service cosmicSession
  cosmicToService : Transition cosmicSession service
  serviceToKernel : Transition service kernel

deniedCannotEnterCosmic : Not (Transition denied cosmicSession)
deniedCannotEnterCosmic ()

deniedCannotEnterService : Not (Transition denied service)
deniedCannotEnterService ()

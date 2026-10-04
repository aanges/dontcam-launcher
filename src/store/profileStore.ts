import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import type { Profile, ModEntry, ResourcePackEntry } from '../types'
import { profileApi } from '../tauri/api'

interface ProfileState {
  profiles: Profile[]
  selectedProfile: Profile | null
  isLoading: boolean
  loadProfiles: () => Promise<void>
  createProfile: (name: string, versionId: string, accountId?: string) => Promise<Profile>
  updateProfile: (profile: Profile) => Promise<void>
  deleteProfile: (profileId: string) => Promise<void>
  duplicateProfile: (profileId: string, newName: string) => Promise<Profile>
  selectProfile: (profile: Profile | null) => void
  addMod: (profileId: string, mod: ModEntry) => Promise<void>
  removeMod: (profileId: string, modId: string) => Promise<void>
  toggleMod: (profileId: string, modId: string) => Promise<void>
  addResourcePack: (profileId: string, pack: ResourcePackEntry) => Promise<void>
  removeResourcePack: (profileId: string, packId: string) => Promise<void>
  reorderResourcePacks: (profileId: string, packIds: string[]) => Promise<void>
}

export const useProfileStore = create<ProfileState>()(
  persist(
    (set, get) => ({
      profiles: [],
      selectedProfile: null,
      isLoading: false,
      
      loadProfiles: async () => {
        set({ isLoading: true })
        try {
          const profiles = (await profileApi.getAll()).map((p) => ({
            ...p,
            dontcam_mod: p.dontcam_mod ?? true,
          }))
          set({ profiles, isLoading: false })
        } catch {
          set({ isLoading: false })
        }
      },
      
      createProfile: async (name, versionId, accountId) => {
        const profile = await profileApi.create(name, versionId, accountId)
        set((state) => ({ profiles: [...state.profiles, profile] }))
        return profile
      },
      
      updateProfile: async (profile) => {
        await profileApi.update(profile)
        set((state) => ({
          profiles: state.profiles.map((p) => (p.id === profile.id ? profile : p)),
          selectedProfile: state.selectedProfile?.id === profile.id ? profile : state.selectedProfile,
        }))
      },
      
      deleteProfile: async (profileId) => {
        await profileApi.delete(profileId)
        set((state) => ({
          profiles: state.profiles.filter((p) => p.id !== profileId),
          selectedProfile: state.selectedProfile?.id === profileId ? null : state.selectedProfile,
        }))
      },
      
      duplicateProfile: async (profileId, newName) => {
        const profile = await profileApi.duplicate(profileId, newName)
        set((state) => ({ profiles: [...state.profiles, profile] }))
        return profile
      },
      
      selectProfile: (profile) => set({ selectedProfile: profile }),
      
      addMod: async (profileId, mod) => {
        const profile = get().profiles.find((p) => p.id === profileId)
        if (!profile) return
        
        const updated = { ...profile, mods: [...profile.mods, mod] }
        await get().updateProfile(updated)
      },
      
      removeMod: async (profileId, modId) => {
        const profile = get().profiles.find((p) => p.id === profileId)
        if (!profile) return
        
        const updated = { ...profile, mods: profile.mods.filter((m) => m.id !== modId) }
        await get().updateProfile(updated)
      },
      
      toggleMod: async (profileId, modId) => {
        const profile = get().profiles.find((p) => p.id === profileId)
        if (!profile) return
        
        const updated = {
          ...profile,
          mods: profile.mods.map((m) => 
            m.id === modId ? { ...m, enabled: !m.enabled } : m
          ),
        }
        await get().updateProfile(updated)
      },
      
      addResourcePack: async (profileId, pack) => {
        const profile = get().profiles.find((p) => p.id === profileId)
        if (!profile) return
        
        const updated = { 
          ...profile, 
          resource_packs: [...profile.resource_packs, { ...pack, priority: profile.resource_packs.length }] 
        }
        await get().updateProfile(updated)
      },
      
      removeResourcePack: async (profileId, packId) => {
        const profile = get().profiles.find((p) => p.id === profileId)
        if (!profile) return
        
        const updated = { 
          ...profile, 
          resource_packs: profile.resource_packs.filter((p) => p.id !== packId) 
        }
        await get().updateProfile(updated)
      },
      
      reorderResourcePacks: async (profileId, packIds) => {
        const profile = get().profiles.find((p) => p.id === profileId)
        if (!profile) return
        
        const packMap = new Map(profile.resource_packs.map((p) => [p.id, p]))
        const reordered = packIds.map((id, index) => ({ ...packMap.get(id)!, priority: index }))
        
        const updated = { ...profile, resource_packs: reordered }
        await get().updateProfile(updated)
      },
    }),
    {
      name: 'profile-storage',
      partialize: (state) => ({ 
        profiles: state.profiles,
        selectedProfile: state.selectedProfile,
      }),
    }
  )
)
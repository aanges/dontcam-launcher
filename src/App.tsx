import { useEffect, useState } from 'react'
import { Routes, Route, Navigate } from 'react-router-dom'
import { Layout } from './components/layout/Layout'
import { OnboardingModal } from './components/OnboardingModal'
import { HomePage } from './pages/HomePage'
import { VersionsPage } from './pages/VersionsPage'
import { ProfilesPage } from './pages/ProfilesPage'
import { AccountsPage } from './pages/AccountsPage'
import { SettingsPage } from './pages/SettingsPage'
import { useAuthStore } from './store/authStore'
import { useProfileStore } from './store/profileStore'
import { useSettingsStore } from './store/settingsStore'
import { useVersionStore } from './store/versionStore'
import { useUpdateStore } from './store/updateStore'
import { initLauncherEvents } from './store/consoleStore'
import { UpdateDialog } from './components/UpdateDialog'

export default function App() {
  const loadAccounts = useAuthStore((s) => s.loadAccounts)
  const loadProfiles = useProfileStore((s) => s.loadProfiles)
  const loadSettings = useSettingsStore((s) => s.loadSettings)
  const loadManifest = useVersionStore((s) => s.loadManifest)
  const loadInstalledVersions = useVersionStore((s) => s.loadInstalledVersions)
  const checkForUpdates = useUpdateStore((s) => s.checkForUpdates)
  const accounts = useAuthStore((s) => s.accounts)
  const authLoading = useAuthStore((s) => s.isLoading)
  const [onboardingDismissed, setOnboardingDismissed] = useState(false)

  useEffect(() => {
    initLauncherEvents()
    loadAccounts()
    loadProfiles()
    loadSettings()
    loadManifest()
    loadInstalledVersions()
    // Silent update check at startup — dialog appears only when an update exists.
    checkForUpdates(true).catch(() => undefined)
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  // First run: no accounts at all -> ask for one. Closes itself on login.
  const showOnboarding = accounts.length === 0 && !authLoading && !onboardingDismissed

  return (
    <>
      <Routes>
        <Route element={<Layout />}>
          <Route path="/" element={<HomePage />} />
          <Route path="/versions" element={<VersionsPage />} />
          <Route path="/profiles" element={<ProfilesPage />} />
          <Route path="/accounts" element={<AccountsPage />} />
          <Route path="/settings" element={<SettingsPage />} />
          <Route path="*" element={<Navigate to="/" replace />} />
        </Route>
      </Routes>
      <OnboardingModal open={showOnboarding} onClose={() => setOnboardingDismissed(true)} />
      <UpdateDialog />
    </>
  )
}

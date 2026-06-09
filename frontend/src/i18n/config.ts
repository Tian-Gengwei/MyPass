import i18n from 'i18next'
import { initReactI18next } from 'react-i18next'
import LanguageDetector from 'i18next-browser-languagedetector'

const resources = {
  en: {
    translation: {
      app: {
        title: 'MyPass',
        subtitle: 'Secure password manager',
        loading: 'Loading...'
      },
      vault: {
        select: 'Select Vault',
        create: 'Create Vault',
        unlock: 'Unlock Vault',
        name: 'Vault Name',
        masterPassword: 'Master Password',
        confirmPassword: 'Confirm Password',
        noVaults: 'No vaults found',
        createFirst: 'Create your first vault to get started',
        entries: '{{count}} entries',
        groups: '{{count}} groups',
        newVault: 'New Vault',
        back: 'Back',
        cancel: 'Cancel',
        unlocking: 'Unlocking...',
        creating: 'Creating...',
        dontHave: "Don't have a vault? Create one",
        alreadyHave: 'Already have a vault? Unlock it',
        selectVault: 'Select a vault to unlock',
        enterPassword: 'Enter your master password to unlock',
        storageLocation: 'Storage Location',
        storageLocationDesc: 'Default folder for storing new vaults',
        useCustomLocation: 'Use custom location',
        customLocation: 'Custom location',
        customLocationPlaceholder: 'Select or enter a folder path',
        browse: 'Browse...',
        browseFolder: 'Choose folder',
        locationReset: 'Reset to default',
        locationUpdated: 'Storage location updated',
        locationResetSuccess: 'Storage location reset to default',
        useDefaultLocation: 'Use default location',
        defaultLocationLabel: 'Default',
        customLocationLabel: 'Custom',
        locationHelp: 'Vaults are stored as subdirectories under the selected location. Each vault is a folder ending with .vault',
        newVaultWillBeStoredAt: 'New vault will be stored at:',
        willSaveToCustom: 'Save to custom location',
        willSaveToDefault: 'Save to default location'
      },
      errors: {
        passwordMismatch: 'Passwords do not match',
        passwordLength: 'Password must be at least 8 characters',
        vaultNameRequired: 'Vault name is required',
        enterPassword: 'Please enter your password',
        unknown: 'An error occurred',
        invalidPath: 'Invalid path',
        vaultAlreadyExists: 'A vault with this name already exists at the target location',
        pathNotAbsolute: 'Path must be absolute',
        storageLocationFailed: 'Failed to update storage location'
      },
      menu: {
        file: 'File',
        edit: 'Edit',
        view: 'View',
        tools: 'Tools',
        help: 'Help',
        newVault: 'New Vault',
        lockVault: 'Lock Vault',
        import: 'Import',
        export: 'Export',
        settings: 'Settings',
        about: 'About',
        exit: 'Exit'
      },
      common: {
        search: 'Search...',
        add: 'Add',
        edit: 'Edit',
        delete: 'Delete',
        save: 'Save',
        cancel: 'Cancel',
        confirm: 'Confirm',
        ok: 'OK',
        yes: 'Yes',
        no: 'No',
        success: 'Success',
        error: 'Error',
        info: 'Info',
        warning: 'Warning',
        select: 'Select',
        import: 'Import',
        export: 'Export',
        remove: 'Remove',
        loading: 'Loading...',
        reset: 'Reset',
        apply: 'Apply',
        refresh: 'Refresh',
        current: 'Current'
      },
      import: {
        title: 'Import Data',
        selectFormat: 'Select import format',
        selectFile: 'Select file',
        importing: 'Importing...',
        success: 'Import completed successfully',
        failed: 'Import failed'
      },
      export: {
        title: 'Export Data',
        selectFormat: 'Select export format',
        exporting: 'Exporting...',
        success: 'Export completed successfully',
        failed: 'Export failed'
      },
      entry: {
        name: 'Name',
        username: 'Username',
        password: 'Password',
        url: 'URL',
        notes: 'Notes',
        otp: 'TOTP',
        group: 'Group',
        newEntry: 'New Entry',
        editEntry: 'Edit Entry',
        deleteEntry: 'Delete Entry',
        confirmDelete: 'Are you sure you want to delete this entry?',
        copy: 'Copy',
        copied: 'Copied!',
        generatePassword: 'Generate Password',
        showPassword: 'Show Password',
        hidePassword: 'Hide Password'
      },
      group: {
        name: 'Group Name',
        newGroup: 'New Group',
        editGroup: 'Edit Group',
        deleteGroup: 'Delete Group',
        confirmDelete: 'Are you sure you want to delete this group?'
      },
      language: {
        english: 'English',
        chinese: '简体中文'
      },
      settings: {
        title: 'Settings',
        security: 'Security',
        appearance: 'Appearance',
        sync: 'Sync',
        about: 'About',
        general: 'General',
        storage: 'Storage',
        storageDesc: 'Manage where vaults are stored on disk'
      },
      passkey: {
        title: 'Passkeys',
        description: 'Use passkeys or hardware security keys to unlock your vault',
        addPasskey: 'Add Passkey',
        addHardwareKey: 'Add Hardware Key',
        noPasskeys: 'No passkeys configured',
        passkeyAdded: 'Passkey added successfully',
        passkeyRemoved: 'Passkey removed',
        removeConfirm: 'Remove this passkey?',
        unlockWithPasskey: 'Unlock with Passkey',
        authenticatorPlatform: 'Platform Authenticator',
        authenticatorCrossPlatform: 'Hardware Security Key',
        created: 'Created',
        lastUsed: 'Last used'
      },
      security: {
        autoLock: 'Auto-lock',
        autoLockAfter: 'Lock after',
        minutes: 'minutes',
        biometrics: 'Biometrics',
        enableBiometrics: 'Enable biometric unlock',
        pin: 'PIN',
        setPin: 'Set PIN',
        changePin: 'Change PIN',
        pinEnabled: 'PIN unlock enabled',
        passkeyEnabled: 'Passkey unlock enabled'
      }
    }
  },
  zh: {
    translation: {
      app: {
        title: 'MyPass',
        subtitle: '安全密码管理器',
        loading: '加载中...'
      },
      vault: {
        select: '选择金库',
        create: '创建金库',
        unlock: '解锁金库',
        name: '金库名称',
        masterPassword: '主密码',
        confirmPassword: '确认密码',
        noVaults: '未找到金库',
        createFirst: '创建您的第一个金库开始使用',
        entries: '{{count}} 个条目',
        groups: '{{count}} 个分组',
        newVault: '新建金库',
        back: '返回',
        cancel: '取消',
        unlocking: '解锁中...',
        creating: '创建中...',
        dontHave: '没有金库？创建一个',
        alreadyHave: '已有金库？解锁它',
        selectVault: '选择要解锁的金库',
        enterPassword: '输入您的主密码以解锁',
        storageLocation: '存储位置',
        storageLocationDesc: '用于存放新金库的默认文件夹',
        useCustomLocation: '使用自定义位置',
        customLocation: '自定义位置',
        customLocationPlaceholder: '选择或输入文件夹路径',
        browse: '浏览...',
        browseFolder: '选择文件夹',
        locationReset: '恢复默认',
        locationUpdated: '存储位置已更新',
        locationResetSuccess: '存储位置已恢复为默认值',
        useDefaultLocation: '使用默认位置',
        defaultLocationLabel: '默认',
        customLocationLabel: '自定义',
        locationHelp: '金库以子目录形式保存在所选位置下，每个金库是一个以 .vault 结尾的文件夹',
        newVaultWillBeStoredAt: '新金库将保存在：',
        willSaveToCustom: '保存到自定义位置',
        willSaveToDefault: '保存到默认位置'
      },
      errors: {
        passwordMismatch: '密码不匹配',
        passwordLength: '密码至少需要8个字符',
        vaultNameRequired: '金库名称不能为空',
        enterPassword: '请输入密码',
        unknown: '发生未知错误',
        invalidPath: '路径无效',
        vaultAlreadyExists: '目标位置已存在同名金库',
        pathNotAbsolute: '路径必须是绝对路径',
        storageLocationFailed: '更新存储位置失败'
      },
      menu: {
        file: '文件',
        edit: '编辑',
        view: '视图',
        tools: '工具',
        help: '帮助',
        newVault: '新建金库',
        lockVault: '锁定金库',
        import: '导入',
        export: '导出',
        settings: '设置',
        about: '关于',
        exit: '退出'
      },
      common: {
        search: '搜索...',
        add: '添加',
        edit: '编辑',
        delete: '删除',
        save: '保存',
        cancel: '取消',
        confirm: '确认',
        ok: '确定',
        yes: '是',
        no: '否',
        success: '成功',
        error: '错误',
        info: '信息',
        warning: '警告',
        select: '选择',
        import: '导入',
        export: '导出',
        remove: '移除',
        loading: '加载中...',
        reset: '重置',
        apply: '应用',
        refresh: '刷新',
        current: '当前'
      },
      import: {
        title: '导入数据',
        selectFormat: '选择导入格式',
        selectFile: '选择文件',
        importing: '导入中...',
        success: '导入成功完成',
        failed: '导入失败'
      },
      export: {
        title: '导出数据',
        selectFormat: '选择导出格式',
        exporting: '导出中...',
        success: '导出成功完成',
        failed: '导出失败'
      },
      entry: {
        name: '名称',
        username: '用户名',
        password: '密码',
        url: '网址',
        notes: '备注',
        otp: 'TOTP',
        group: '分组',
        newEntry: '新建条目',
        editEntry: '编辑条目',
        deleteEntry: '删除条目',
        confirmDelete: '确定要删除此条目吗？',
        copy: '复制',
        copied: '已复制！',
        generatePassword: '生成密码',
        showPassword: '显示密码',
        hidePassword: '隐藏密码'
      },
      group: {
        name: '分组名称',
        newGroup: '新建分组',
        editGroup: '编辑分组',
        deleteGroup: '删除分组',
        confirmDelete: '确定要删除此分组吗？'
      },
      language: {
        english: 'English',
        chinese: '简体中文'
      },
      settings: {
        title: '设置',
        security: '安全',
        appearance: '外观',
        sync: '同步',
        about: '关于',
        general: '通用',
        storage: '存储',
        storageDesc: '管理金库在磁盘上的保存位置'
      },
      passkey: {
        title: '通行密钥',
        description: '使用通行密钥或硬件安全密钥解锁您的金库',
        addPasskey: '添加通行密钥',
        addHardwareKey: '添加硬件密钥',
        noPasskeys: '未配置通行密钥',
        passkeyAdded: '通行密钥添加成功',
        passkeyRemoved: '通行密钥已移除',
        removeConfirm: '确定要移除此通行密钥吗？',
        unlockWithPasskey: '使用通行密钥解锁',
        authenticatorPlatform: '平台认证器',
        authenticatorCrossPlatform: '硬件安全密钥',
        created: '创建时间',
        lastUsed: '最后使用'
      },
      security: {
        autoLock: '自动锁定',
        autoLockAfter: '锁定时间',
        minutes: '分钟',
        biometrics: '生物识别',
        enableBiometrics: '启用生物识别解锁',
        pin: 'PIN码',
        setPin: '设置PIN码',
        changePin: '修改PIN码',
        pinEnabled: 'PIN码解锁已启用',
        passkeyEnabled: '通行密钥解锁已启用'
      }
    }
  }
}

i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    resources,
    fallbackLng: 'zh',
    debug: false,
    interpolation: {
      escapeValue: false
    },
    detection: {
      order: ['querystring', 'cookie', 'localStorage', 'navigator', 'htmlTag', 'path', 'subdomain'],
      caches: ['localStorage', 'cookie']
    }
  })

export default i18n
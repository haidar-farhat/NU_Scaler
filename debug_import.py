try:
    import nu_scaler_core
    print('nu_scaler_core imported successfully')
    print('PyDlssUpscaler' in dir(nu_scaler_core))
    print(dir(nu_scaler_core))
except Exception as e:
    print(f'Import failed: {e}')
    import traceback
    traceback.print_exc() 
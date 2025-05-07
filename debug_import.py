try:
    import nu_scaler
    print('nu_scaler imported successfully')
    print('PyDlssUpscaler' in dir(nu_scaler))
    print(dir(nu_scaler))
except Exception as e:
    print(f'Import failed: {e}')
    import traceback
    traceback.print_exc()
import sys; sys.stdout.flush() 
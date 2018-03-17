namespace QCExtensions.Server.Models.ViewModels.Results
{
	public class ErrorMessageViewModel : ResultViewModelBase {
		public ErrorMessageViewModel(string message)
		{
			Success = false;
			Message = message;
		}

	}
}

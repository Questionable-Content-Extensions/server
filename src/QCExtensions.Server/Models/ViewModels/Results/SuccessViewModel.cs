namespace QCExtensions.Server.Models.ViewModels.Results
{
	public class SuccessViewModel : ResultViewModelBase
	{
		public SuccessViewModel(string message)
		{
			Success = true;
			Message = message;
		}
	}
}

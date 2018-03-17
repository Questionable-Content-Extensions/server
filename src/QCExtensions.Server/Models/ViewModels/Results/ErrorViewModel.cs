namespace QCExtensions.Server.Models.ViewModels.Results
{
	public class ErrorViewModel : ResultViewModelBase
	{
		public ErrorViewModel(params AssociatedError[] errors)
		{
			Success = false;
			Errors.AddRange(errors);
		}
	}
}

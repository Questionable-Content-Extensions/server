namespace QCExtensions.Server.Models.ViewModels.Results
{
	public class SuccessObjectViewModel : SuccessViewModel
	{
		public object Data { get; set; }

		public SuccessObjectViewModel(object data)
			: base(null)
		{
			Data = data;
		}

		public SuccessObjectViewModel(string message, object data)
			: base(message)
		{
			Data = data;
		}
	}
}

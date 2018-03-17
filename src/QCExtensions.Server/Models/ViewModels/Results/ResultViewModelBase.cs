using System.Collections.Generic;
using System.Linq;

namespace QCExtensions.Server.Models.ViewModels.Results
{
	public abstract class ResultViewModelBase
	{
		public abstract class ErrorBase
		{
		}

		public class AssociatedError : ErrorBase
		{
			public string Association { get; private set; }
			public List<string> ErrorMessages { get; private set; }

			public AssociatedError(string association, params string[] errorMessages)
			{
				Association = association;
				ErrorMessages = errorMessages.ToList();
			}
		}

		public bool Success { get; set; }
		public List<ErrorBase> Errors { get; private set; }
		public string Message { get; set; }

		public ResultViewModelBase()
		{
			Errors = new List<ErrorBase>();
		}
	}
}
